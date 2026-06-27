use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::cookie::{CookieStore, Jar};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, COOKIE};
use reqwest::redirect::Policy;
use url::Url;

use crate::application::error::{AppError, AppResult};
use crate::infrastructure::network::models::HttpResponseData;
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct HttpTransport {
    settings: AppSettings,
    source_ip: Option<IpAddr>,
    clients: Arc<Mutex<HashMap<ClientKey, reqwest::Client>>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ClientKey {
    use_source_ip: bool,
    max_redirects: usize,
}

#[derive(Clone, Debug)]
pub struct HttpRequestSpec {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub cookies: HashMap<String, String>,
    pub max_redirects: usize,
    pub preserve_redirect_cookies: bool,
}

impl HttpRequestSpec {
    pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            url: url.into(),
            headers: HashMap::new(),
            body: String::new(),
            cookies: HashMap::new(),
            max_redirects: 5,
            preserve_redirect_cookies: false,
        }
    }

    pub fn get(url: impl Into<String>) -> Self {
        Self::new("GET", url)
    }

    pub fn post(url: impl Into<String>) -> Self {
        Self::new("POST", url)
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub fn cookies(mut self, cookies: HashMap<String, String>) -> Self {
        self.cookies = cookies;
        self
    }

    pub fn max_redirects(mut self, max_redirects: usize) -> Self {
        self.max_redirects = max_redirects;
        self
    }

    pub fn preserve_redirect_cookies(mut self) -> Self {
        self.preserve_redirect_cookies = true;
        self
    }
}

impl HttpTransport {
    pub fn new(settings: AppSettings) -> AppResult<Self> {
        let source_ip_client = if settings.bind_preferred_source_ip
            && !settings.preferred_source_ip.trim().is_empty()
        {
            settings.preferred_source_ip.parse::<IpAddr>().ok()
        } else {
            None
        };
        Ok(Self {
            settings,
            source_ip: source_ip_client,
            clients: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn request(&self, spec: HttpRequestSpec) -> AppResult<HttpResponseData> {
        let modes = if self.source_ip.is_some() {
            vec![true, false]
        } else {
            vec![false]
        };

        let mut last_error = None;
        for use_source_ip in modes {
            match self.request_with_mode(&spec, use_source_ip).await {
                Ok(response) => return Ok(response),
                Err(err) => last_error = Some(err),
            }
        }

        Err(AppError::Network(format!(
            "HTTP 请求失败，interface={}, preferred_source_ip={}, bind_enabled={}, url={}, error={}",
            self.settings.preferred_interface_name,
            if self.settings.bind_preferred_source_ip { self.settings.preferred_source_ip.as_str() } else { "disabled" },
            self.settings.bind_preferred_source_ip,
            spec.url,
            last_error.map(|err| err.to_string()).unwrap_or_else(|| "unknown".to_string())
        )))
    }

    async fn request_with_mode(
        &self,
        spec: &HttpRequestSpec,
        use_source_ip: bool,
    ) -> AppResult<HttpResponseData> {
        let parsed_url = Url::parse(&spec.url)
            .map_err(|err| AppError::Network(format!("无效 URL：{}，{err}", spec.url)))?;
        let cookie_jar = if spec.preserve_redirect_cookies {
            let jar = Arc::new(Jar::default());
            for (name, value) in &spec.cookies {
                jar.add_cookie_str(&format!("{name}={value}"), &parsed_url);
            }
            Some(jar)
        } else {
            None
        };
        let client = if let Some(cookie_jar) = cookie_jar.as_ref() {
            build_client(
                if use_source_ip { self.source_ip } else { None },
                spec.max_redirects,
                Some(cookie_jar.clone()),
            )?
        } else {
            self.client_for(use_source_ip, spec.max_redirects)?
        };
        let method = spec
            .method
            .parse::<reqwest::Method>()
            .map_err(|err| AppError::Network(format!("无效 HTTP 方法：{}，{err}", spec.method)))?;
        let mut request = client.request(method, parsed_url.clone());

        let mut header_map = HeaderMap::new();
        for (key, value) in &spec.headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|err| AppError::Network(format!("无效请求头：{key}，{err}")))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|err| AppError::Network(format!("无效请求头值：{key}，{err}")))?;
            header_map.insert(header_name, header_value);
        }
        if !spec.cookies.is_empty() {
            let cookie_text = spec
                .cookies
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("; ");
            header_map.insert(
                COOKIE,
                HeaderValue::from_str(&cookie_text)
                    .unwrap_or_else(|_| HeaderValue::from_static("")),
            );
        }
        request = request.headers(header_map);
        if !spec.body.is_empty() {
            request = request.body(spec.body.clone());
        }

        let response = request.send().await?;
        let status = response.status();
        let final_url = response.url().to_string();
        let reason = status.canonical_reason().unwrap_or("").to_string();
        let headers = response.headers().clone();
        let raw_body = response.bytes().await?.to_vec();
        let text = String::from_utf8_lossy(&raw_body).to_string();

        if status.is_client_error() || status.is_server_error() {
            return Err(AppError::Network(format!(
                "请求失败: status={}, reason={}, url={}, body={}",
                status.as_u16(),
                reason,
                final_url,
                text.chars().take(200).collect::<String>()
            )));
        }

        let mut cookie_map = cookie_jar
            .as_ref()
            .map(|jar| extract_cookie_map(jar.as_ref(), &parsed_url, &spec.cookies))
            .unwrap_or_else(|| spec.cookies.clone());
        for value in headers.get_all("set-cookie").iter() {
            if let Ok(raw) = value.to_str() {
                if let Some((name, rest)) = raw.split_once('=') {
                    let cookie_value = rest.split(';').next().unwrap_or_default().to_string();
                    cookie_map.insert(name.trim().to_string(), cookie_value);
                }
            }
        }

        Ok(HttpResponseData {
            final_url,
            status: status.as_u16(),
            reason,
            raw_body,
            text,
            cookies: cookie_map,
        })
    }

    fn client_for(&self, use_source_ip: bool, max_redirects: usize) -> AppResult<reqwest::Client> {
        let key = ClientKey {
            use_source_ip,
            max_redirects,
        };
        let mut clients = self
            .clients
            .lock()
            .map_err(|_| AppError::Internal("HTTP client cache lock poisoned".to_string()))?;
        if let Some(client) = clients.get(&key) {
            return Ok(client.clone());
        }
        let source_ip = if use_source_ip { self.source_ip } else { None };
        let client = build_client(source_ip, max_redirects, None)?;
        clients.insert(key, client.clone());
        Ok(client)
    }
}

fn build_client(
    source_ip: Option<IpAddr>,
    max_redirects: usize,
    cookie_jar: Option<Arc<Jar>>,
) -> AppResult<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .redirect(Policy::limited(max_redirects))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0");

    if let Some(cookie_jar) = cookie_jar {
        builder = builder.cookie_provider(cookie_jar);
    }

    if let Some(ip) = source_ip {
        builder = builder.local_address(ip);
    }

    Ok(builder.build()?)
}

fn extract_cookie_map(
    cookie_jar: &Jar,
    url: &Url,
    fallback: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut cookie_map = fallback.clone();
    let Some(cookie_header) = cookie_jar.cookies(url) else {
        return cookie_map;
    };
    let Ok(cookie_text) = cookie_header.to_str() else {
        return cookie_map;
    };
    for part in cookie_text.split(';') {
        let Some((name, value)) = part.trim().split_once('=') else {
            continue;
        };
        cookie_map.insert(name.trim().to_string(), value.trim().to_string());
    }
    cookie_map
}

pub fn build_form_headers(referer_url: &str) -> HashMap<String, String> {
    let origin = url::Url::parse(referer_url)
        .ok()
        .and_then(|url| Some(format!("{}://{}", url.scheme(), url.host_str()?)))
        .unwrap_or_default();
    HashMap::from([
        (
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded; charset=UTF-8".to_string(),
        ),
        ("Origin".to_string(), origin),
        ("Referer".to_string(), referer_url.to_string()),
    ])
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::{HttpRequestSpec, HttpTransport};
    use crate::infrastructure::settings::AppSettings;

    #[tokio::test]
    async fn keeps_set_cookie_across_redirects() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sso"))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("set-cookie", "PHPSESSID_8800=abc; path=/; HttpOnly")
                    .insert_header("location", "/home"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/home"))
            .respond_with(|request: &wiremock::Request| {
                let cookie = request
                    .headers
                    .get("cookie")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default();
                if cookie.contains("PHPSESSID_8800=abc") {
                    ResponseTemplate::new(200).set_body_string("home")
                } else {
                    ResponseTemplate::new(200).set_body_string("login")
                }
            })
            .mount(&server)
            .await;

        let transport = HttpTransport::new(AppSettings::default()).expect("create transport");
        let response = transport
            .request(HttpRequestSpec::get(format!("{}/sso", server.uri())).max_redirects(5))
            .await
            .expect("request sso");

        assert_eq!(response.text, "login");
    }

    #[tokio::test]
    async fn keeps_set_cookie_across_redirects_when_enabled() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sso"))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("set-cookie", "PHPSESSID_8800=abc; path=/; HttpOnly")
                    .insert_header("location", "/home"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/home"))
            .respond_with(|request: &wiremock::Request| {
                let cookie = request
                    .headers
                    .get("cookie")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default();
                if cookie.contains("PHPSESSID_8800=abc") {
                    ResponseTemplate::new(200).set_body_string("home")
                } else {
                    ResponseTemplate::new(200).set_body_string("login")
                }
            })
            .mount(&server)
            .await;

        let transport = HttpTransport::new(AppSettings::default()).expect("create transport");
        let response = transport
            .request(
                HttpRequestSpec::get(format!("{}/sso", server.uri()))
                    .max_redirects(5)
                    .preserve_redirect_cookies(),
            )
            .await
            .expect("request sso");

        assert_eq!(response.text, "home");
        assert_eq!(
            response.cookies.get("PHPSESSID_8800").map(String::as_str),
            Some("abc")
        );
    }
}
