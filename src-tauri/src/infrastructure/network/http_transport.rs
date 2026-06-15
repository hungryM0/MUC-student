use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

use base64::Engine;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, COOKIE};
use reqwest::redirect::Policy;
use url::Url;

use crate::application::error::{AppError, AppResult};
use crate::infrastructure::network::models::HttpResponseData;
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct HttpTransport {
    settings: AppSettings,
}

impl HttpTransport {
    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
    }

    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: String,
        cookies: HashMap<String, String>,
        max_redirects: usize,
    ) -> AppResult<HttpResponseData> {
        let modes = if self.settings.bind_preferred_source_ip
            && !self.settings.preferred_source_ip.trim().is_empty()
        {
            vec![true, false]
        } else {
            vec![false]
        };

        let mut last_error = None;
        for use_source_ip in modes {
            match self
                .request_with_mode(
                    method,
                    url,
                    &headers,
                    &body,
                    &cookies,
                    max_redirects,
                    use_source_ip,
                )
                .await
            {
                Ok(response) => return Ok(response),
                Err(err) => last_error = Some(err),
            }
        }

        Err(AppError::Network(format!(
            "HTTP 请求失败，interface={}, preferred_source_ip={}, bind_enabled={}, url={}, error={}",
            self.settings.preferred_interface_name,
            if self.settings.bind_preferred_source_ip { self.settings.preferred_source_ip.as_str() } else { "disabled" },
            self.settings.bind_preferred_source_ip,
            url,
            last_error.map(|err| err.to_string()).unwrap_or_else(|| "unknown".to_string())
        )))
    }

    async fn request_with_mode(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: &str,
        cookies: &HashMap<String, String>,
        max_redirects: usize,
        use_source_ip: bool,
    ) -> AppResult<HttpResponseData> {
        let mut builder = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(12))
            .redirect(Policy::limited(max_redirects))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0");

        if use_source_ip {
            if let Ok(ip) = self.settings.preferred_source_ip.parse::<IpAddr>() {
                builder = builder.local_address(ip);
            }
        }

        let client = builder.build()?;
        let parsed_url =
            Url::parse(url).map_err(|err| AppError::Network(format!("无效 URL：{url}，{err}")))?;
        let method = method
            .parse::<reqwest::Method>()
            .map_err(|err| AppError::Network(format!("无效 HTTP 方法：{method}，{err}")))?;
        let mut request = client.request(method, parsed_url);

        let mut header_map = HeaderMap::new();
        for (key, value) in headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())
                .map_err(|err| AppError::Network(format!("无效请求头：{key}，{err}")))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|err| AppError::Network(format!("无效请求头值：{key}，{err}")))?;
            header_map.insert(header_name, header_value);
        }
        if !cookies.is_empty() {
            let cookie_text = cookies
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
        if !body.is_empty() {
            request = request.body(body.to_string());
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

        let mut cookie_map = cookies.clone();
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
}

pub fn encode_password(password: &str) -> String {
    format!(
        "{{B}}{}",
        base64::engine::general_purpose::STANDARD.encode(password.as_bytes())
    )
}
