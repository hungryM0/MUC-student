use std::collections::HashMap;
use std::time::Duration;

use base64::Engine;

use crate::application::error::{AppError, AppResult};
use crate::infrastructure::network::http_transport::{
    build_form_headers, HttpRequestSpec, HttpTransport,
};
use crate::infrastructure::network::models::HttpResponseData;
use crate::infrastructure::parsers::online_device_parser::parse_online_devices;
use crate::infrastructure::parsers::panel_home_parser::extract_csrf_meta;
use crate::infrastructure::parsers::portal_page_parser::{is_traffic_home_page, join_url};
use crate::infrastructure::persistence::account_repository::AccountWithPassword;
use crate::infrastructure::persistence::panel_session_repository::PanelSessionRepository;
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct SelfServicePanelClient {
    settings: AppSettings,
    transport: HttpTransport,
    session_repo: PanelSessionRepository,
}

impl SelfServicePanelClient {
    const LOCAL_DEVICE_VERIFY_RETRY_DELAYS_MS: [u64; 3] = [0, 600, 1200];

    pub fn new(
        settings: AppSettings,
        transport: HttpTransport,
        session_repo: PanelSessionRepository,
    ) -> Self {
        Self {
            settings,
            transport,
            session_repo,
        }
    }

    pub async fn fetch_authenticated_html(
        &self,
        account: &AccountWithPassword,
        path: &str,
    ) -> AppResult<String> {
        Ok(self.fetch_authenticated_page(account, path).await?.text)
    }

    pub async fn fetch_authenticated_page(
        &self,
        account: &AccountWithPassword,
        path: &str,
    ) -> AppResult<HttpResponseData> {
        let target_url = self.traffic_entry_url();
        let target_path = if path.trim().is_empty() {
            "/home"
        } else {
            path.trim()
        };

        let saved_cookies = self
            .session_repo
            .load_session(&account.account.id)?
            .into_iter()
            .collect::<HashMap<_, _>>();
        if !saved_cookies.is_empty() {
            let session_response = self
                .transport
                .request(
                    HttpRequestSpec::get(join_url(&target_url, target_path))
                        .cookies(saved_cookies)
                        .max_redirects(5),
                )
                .await?;
            if is_traffic_home_page(&session_response.text) {
                self.persist_session(account, &session_response.cookies)?;
                return Ok(session_response);
            }
            if !is_login_page(&session_response.text) {
                self.persist_session(account, &session_response.cookies)?;
                return Ok(session_response);
            }
            self.session_repo.clear_session(&account.account.id)?;
        }

        if let Some(response) = self.open_with_sso(account, target_path).await? {
            self.persist_session(account, &response.cookies)?;
            return Ok(response);
        }

        let page_response = self
            .transport
            .request(HttpRequestSpec::get(&target_url).max_redirects(5))
            .await?;
        if is_traffic_home_page(&page_response.text) {
            self.persist_session(account, &page_response.cookies)?;
            return Ok(page_response);
        }
        let retry_response = self
            .transport
            .request(
                HttpRequestSpec::get(join_url(&page_response.final_url, target_path))
                    .cookies(page_response.cookies.clone())
                    .max_redirects(5),
            )
            .await?;
        if is_traffic_home_page(&retry_response.text) {
            self.persist_session(account, &retry_response.cookies)?;
            return Ok(retry_response);
        }
        if is_login_page(&retry_response.text) {
            return Err(AppError::Network(
                "登录态失效，访问目标页面时被重定向回登录页".to_string(),
            ));
        }
        Err(AppError::Network(format!(
            "流量入口不匹配：query_url={}, final_url={}",
            self.settings.traffic_portal_url, page_response.final_url
        )))
    }

    async fn open_with_sso(
        &self,
        account: &AccountWithPassword,
        target_path: &str,
    ) -> AppResult<Option<HttpResponseData>> {
        let sso_url = build_sso_url(&self.traffic_entry_url(), &account.account.username);
        let response = self
            .transport
            .request(HttpRequestSpec::get(sso_url).max_redirects(5))
            .await?;
        if is_login_page(&response.text) {
            return Ok(None);
        }
        if is_traffic_home_page(&response.text) {
            return self.fetch_target_after_login(response, target_path).await;
        }

        let retry_response = self
            .transport
            .request(
                HttpRequestSpec::get(join_url(&response.final_url, target_path))
                    .cookies(response.cookies.clone())
                    .max_redirects(5),
            )
            .await?;
        if is_login_page(&retry_response.text) {
            return Ok(None);
        }
        if is_traffic_home_page(&retry_response.text) {
            return Ok(Some(retry_response));
        }
        Ok(None)
    }

    async fn fetch_target_after_login(
        &self,
        mut response: HttpResponseData,
        target_path: &str,
    ) -> AppResult<Option<HttpResponseData>> {
        let response_path = url::Url::parse(&response.final_url)
            .ok()
            .map(|url| url.path().trim_end_matches('/').to_string())
            .unwrap_or_default();
        if response_path == target_path.trim_end_matches('/') {
            return Ok(Some(response));
        }

        response = self
            .transport
            .request(
                HttpRequestSpec::get(join_url(&response.final_url, target_path))
                    .cookies(response.cookies.clone())
                    .max_redirects(5),
            )
            .await?;
        if is_login_page(&response.text) {
            return Ok(None);
        }
        if is_traffic_home_page(&response.text) {
            return Ok(Some(response));
        }
        Err(AppError::Network(format!(
            "访问 {target_path} 成功，但页面里没有流量表格"
        )))
    }

    pub async fn logout_local_device(
        &self,
        account: &AccountWithPassword,
        local_ip: &str,
    ) -> AppResult<String> {
        let local_ip = local_ip.trim();
        if local_ip.is_empty() || local_ip == "unknown" {
            return Err(AppError::Validation(
                "本机 IP 未知，无法执行本机下线".to_string(),
            ));
        }

        let home_response = self.fetch_authenticated_page(account, "/home").await?;
        let (csrf_param, csrf_token) = extract_csrf_meta(&home_response.text);
        if csrf_param.is_empty() || csrf_token.is_empty() {
            return Err(AppError::Network(
                "当前账号 /home 页面缺少 CSRF 字段，无法执行本机下线".to_string(),
            ));
        }
        let Some(local_device) = parse_online_devices(&home_response.text)
            .into_iter()
            .find(|record| record.ip.trim() == local_ip)
        else {
            return Err(AppError::NotFound(format!(
                "当前账号在线信息里没找到本机 IP：{local_ip}"
            )));
        };
        if !local_device.logout_path.starts_with("/home/delete") {
            return Err(AppError::Network(
                "在线设备下线链接不是 /home/delete，拒绝混用接口".to_string(),
            ));
        }

        let payload = url::form_urlencoded::Serializer::new(String::new())
            .append_pair(&csrf_param, &csrf_token)
            .finish();
        let logout_response = self
            .transport
            .request(
                HttpRequestSpec::post(join_url(
                    &home_response.final_url,
                    &local_device.logout_path,
                ))
                .headers(build_form_headers(&home_response.final_url))
                .body(payload)
                .cookies(home_response.cookies.clone())
                .max_redirects(3),
            )
            .await?;
        self.persist_session(account, &logout_response.cookies)?;

        let verify_url = join_url(&home_response.final_url, "/home");
        let mut verify_cookies = logout_response.cookies;
        for delay_ms in Self::LOCAL_DEVICE_VERIFY_RETRY_DELAYS_MS {
            if delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
            let verify_response = self
                .transport
                .request(
                    HttpRequestSpec::get(&verify_url)
                        .cookies(verify_cookies.clone())
                        .max_redirects(5),
                )
                .await?;
            verify_cookies = verify_response.cookies.clone();
            self.persist_session(account, &verify_cookies)?;
            if !parse_online_devices(&verify_response.text)
                .iter()
                .any(|record| record.ip.trim() == local_ip)
            {
                return Ok(format!(
                    "本机设备下线成功：账号={}，ip={local_ip}",
                    account.account.display_name()
                ));
            }
        }
        Err(AppError::Network(
            "本机设备下线后校验失败：在线列表里仍然存在本机 IP 记录".to_string(),
        ))
    }

    fn persist_session(
        &self,
        account: &AccountWithPassword,
        cookies: &HashMap<String, String>,
    ) -> AppResult<()> {
        self.session_repo.save_session(&account.account.id, cookies)
    }

    fn traffic_entry_url(&self) -> String {
        if self.settings.traffic_portal_url.trim().is_empty() {
            self.settings.portal_url.clone()
        } else {
            self.settings.traffic_portal_url.clone()
        }
    }
}

fn is_login_page(html: &str) -> bool {
    html.contains("LoginForm[username]")
        || html.contains("LoginForm[password]")
        || html.contains("LoginForm[verifyCode]")
        || html.contains("验证码")
        || html.contains("<title>登录</title>")
}

fn build_sso_url(base_url: &str, username: &str) -> String {
    let clean_username = username.trim();
    let data = base64::engine::general_purpose::STANDARD
        .encode(format!("{clean_username}:{clean_username}").as_bytes());
    join_url(base_url, &format!("/site/sso?data={data}"))
}

#[cfg(test)]
mod tests {
    use super::build_sso_url;

    #[test]
    fn sso_url_uses_traffic_portal_origin_and_encoded_username_pair() {
        assert_eq!(
            build_sso_url("http://192.168.2.231:8800/home", "25040034"),
            "http://192.168.2.231:8800/site/sso?data=MjUwNDAwMzQ6MjUwNDAwMzQ="
        );
    }
}
