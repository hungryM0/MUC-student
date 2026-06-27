use std::collections::HashMap;

use base64::Engine;

use crate::application::error::{AppError, AppResult};
use crate::infrastructure::network::http_transport::{HttpRequestSpec, HttpTransport};
use crate::infrastructure::network::models::HttpResponseData;
use crate::infrastructure::parsers::portal_page_parser::{is_traffic_home_page, join_url};
use crate::infrastructure::persistence::panel_session_repository::PanelSessionRepository;
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct SelfServicePanelClient {
    settings: AppSettings,
    transport: HttpTransport,
    session_repo: PanelSessionRepository,
}

impl SelfServicePanelClient {
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

    pub async fn fetch_cached_session_html(
        &self,
        account_id: &str,
        path: &str,
    ) -> AppResult<Option<String>> {
        Ok(self
            .fetch_cached_session_page(account_id, path)
            .await?
            .map(|response| response.text))
    }

    pub async fn fetch_cached_session_page(
        &self,
        account_id: &str,
        path: &str,
    ) -> AppResult<Option<HttpResponseData>> {
        let target_url = self.traffic_entry_url();
        let target_path = if path.trim().is_empty() {
            "/home"
        } else {
            path.trim()
        };
        let saved_cookies = self
            .session_repo
            .load_session(account_id)?
            .into_iter()
            .collect::<HashMap<_, _>>();
        if saved_cookies.is_empty() {
            return Ok(None);
        }

        let response = self
            .transport
            .request(
                HttpRequestSpec::get(join_url(&target_url, target_path))
                    .cookies(saved_cookies)
                    .max_redirects(5)
                    .preserve_redirect_cookies(),
            )
            .await?;
        if is_login_page(&response.text) {
            self.session_repo.clear_session(account_id)?;
            return Ok(None);
        }
        if is_traffic_home_page(&response.text) {
            self.session_repo
                .save_session(account_id, &response.cookies)?;
            return Ok(Some(response));
        }
        Ok(None)
    }

    pub async fn fetch_sso_html(
        &self,
        account_id: &str,
        username: &str,
        path: &str,
    ) -> AppResult<String> {
        Ok(self.fetch_sso_page(account_id, username, path).await?.text)
    }

    pub async fn fetch_sso_page(
        &self,
        account_id: &str,
        username: &str,
        path: &str,
    ) -> AppResult<HttpResponseData> {
        let target_path = if path.trim().is_empty() {
            "/home"
        } else {
            path.trim()
        };
        let response = self
            .open_with_sso_username(username, target_path)
            .await?
            .ok_or_else(|| AppError::Network("自助服务 SSO 没有进入目标页面".to_string()))?;
        self.session_repo
            .save_session(account_id, &response.cookies)?;
        Ok(response)
    }

    async fn open_with_sso_username(
        &self,
        username: &str,
        target_path: &str,
    ) -> AppResult<Option<HttpResponseData>> {
        let sso_url = build_sso_url(&self.traffic_entry_url(), username);
        let response = self
            .transport
            .request(
                HttpRequestSpec::get(sso_url)
                    .max_redirects(5)
                    .preserve_redirect_cookies(),
            )
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
                    .max_redirects(5)
                    .preserve_redirect_cookies(),
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
                    .max_redirects(5)
                    .preserve_redirect_cookies(),
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
