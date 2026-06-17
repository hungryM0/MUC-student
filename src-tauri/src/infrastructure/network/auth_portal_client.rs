use std::collections::HashMap;

use chrono::Local;

use crate::application::error::{AppError, AppResult};
use crate::domain::models::{LoginResult, PortalHiddenFields};
use crate::infrastructure::network::http_transport::{encode_password, HttpTransport};
use crate::infrastructure::network::models::PortalPageData;
use crate::infrastructure::parsers::legacy_portal_success_page_parser::parse_legacy_portal_success_page;
use crate::infrastructure::parsers::portal_page_parser::{
    is_yii_login_page, join_url, parse_hidden_fields,
};
use crate::infrastructure::persistence::account_repository::AccountWithPassword;
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct AuthPortalClient {
    settings: AppSettings,
    transport: HttpTransport,
}

#[derive(Clone, Debug, Default)]
struct SuccessLogoutForm {
    action: String,
    ac_id: String,
    info: String,
    user_ip: String,
    username: String,
}

impl AuthPortalClient {
    const RESPONSE_IP_ALREADY_ONLINE: &'static str = "IP has been online, please logout.";

    pub fn new(settings: AppSettings, transport: HttpTransport) -> Self {
        Self {
            settings,
            transport,
        }
    }

    pub async fn fetch_login_page(&self) -> AppResult<PortalPageData> {
        let response = self
            .transport
            .request(
                "GET",
                &self.settings.portal_url,
                HashMap::new(),
                String::new(),
                HashMap::new(),
                5,
            )
            .await?;
        Ok(PortalPageData {
            login_url: response.final_url.clone(),
            html: response.text.clone(),
            hidden_fields: parse_hidden_fields(&response.text, &response.final_url),
            cookies: response.cookies,
        })
    }

    pub async fn verify_login(&self, account: &AccountWithPassword) -> AppResult<LoginResult> {
        let response = self.login_with_fixed_ac_id(account).await?;

        let response_text = response.text.trim().to_string();
        let already_online = response_text == Self::RESPONSE_IP_ALREADY_ONLINE;
        let success = is_portal_login_success(&response_text);
        let message = if already_online {
            "当前 IP 已在线，无法确认是否为目标账号".to_string()
        } else if success {
            "HTTP 接口登录成功".to_string()
        } else {
            format!(
                "HTTP 接口登录失败：{}",
                if response_text.is_empty() {
                    "服务器未返回内容"
                } else {
                    &response_text
                }
            )
        };

        Ok(LoginResult {
            success,
            message,
            login_url: self.settings.portal_url.clone(),
            hidden_fields: PortalHiddenFields {
                ac_id: "1".to_string(),
                ..Default::default()
            },
            response_text,
            checked_at: Local::now(),
            already_online,
        })
    }

    pub async fn switch_account(
        &self,
        current_account: &AccountWithPassword,
        target_account: &AccountWithPassword,
    ) -> AppResult<LoginResult> {
        let _ = current_account;
        let response = self.login_with_fixed_ac_id(target_account).await?;
        let response_text = response.text.trim().to_string();
        let success = is_portal_login_success(&response_text);

        Ok(LoginResult {
            success,
            message: if success {
                format!(
                    "Portal 切号成功：{} -> {}",
                    current_account.account.display_name(),
                    target_account.account.display_name()
                )
            } else {
                format!(
                    "Portal 切号失败：{}",
                    if response_text.is_empty() {
                        "服务器未返回内容"
                    } else {
                        &response_text
                    }
                )
            },
            login_url: self.settings.portal_url.clone(),
            hidden_fields: PortalHiddenFields {
                ac_id: "1".to_string(),
                ..Default::default()
            },
            response_text,
            checked_at: Local::now(),
            already_online: false,
        })
    }

    pub async fn logout_current_ip(&self, account: &AccountWithPassword) -> AppResult<String> {
        let page_data = self.fetch_login_page().await?;
        if is_yii_login_page(&page_data.html) {
            return Err(AppError::Network(
                "当前 Portal 入口是验证码登录页，轻量链路无法下线".to_string(),
            ));
        }
        if is_legacy_success_page(&page_data.html) {
            return self.logout_with_success_page(account, &page_data).await;
        }
        self.logout_with_page_data(account, &page_data).await
    }

    fn build_login_headers(&self, referer_url: &str) -> HashMap<String, String> {
        let mut headers = self.build_form_headers(referer_url);
        headers.insert("X-Requested-With".to_string(), "XMLHttpRequest".to_string());
        headers
    }

    async fn login_with_fixed_ac_id(
        &self,
        account: &AccountWithPassword,
    ) -> AppResult<crate::infrastructure::network::models::HttpResponseData> {
        let post_url = join_url(&self.settings.portal_url, "/include/auth_action.php");
        let payload = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("action", "login")
            .append_pair("username", &account.account.username)
            .append_pair("password", &encode_password(&account.password))
            .append_pair("ac_id", "1")
            .append_pair("user_ip", "")
            .append_pair("nas_ip", "")
            .append_pair("user_mac", "")
            .append_pair("save_me", "0")
            .append_pair("ajax", "1")
            .finish();
        self.transport
            .request(
                "POST",
                &post_url,
                self.build_login_headers(&self.settings.portal_url),
                payload,
                HashMap::new(),
                1,
            )
            .await
    }

    async fn logout_with_page_data(
        &self,
        account: &AccountWithPassword,
        page_data: &PortalPageData,
    ) -> AppResult<String> {
        let post_url = join_url(&page_data.login_url, "/include/auth_action.php");
        let payload = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("action", "logout")
            .append_pair("username", &account.account.username)
            .append_pair("password", &account.password)
            .append_pair("ajax", "1")
            .finish();
        let response = self
            .transport
            .request(
                "POST",
                &post_url,
                self.build_login_headers(&page_data.login_url),
                payload,
                HashMap::new(),
                1,
            )
            .await?;
        let response_text = response.text.trim().to_string();
        if response_text == "网络已断开" {
            Ok(response_text)
        } else {
            Err(AppError::Network(format!(
                "Portal 下线失败：{}",
                if response_text.is_empty() {
                    "服务器未返回内容"
                } else {
                    &response_text
                }
            )))
        }
    }

    async fn logout_with_success_page(
        &self,
        account: &AccountWithPassword,
        page_data: &PortalPageData,
    ) -> AppResult<String> {
        let form = parse_success_logout_form(&page_data.html);
        let username = if form.username.trim().is_empty() {
            account.account.username.as_str()
        } else {
            form.username.as_str()
        };
        let post_url = join_url(&page_data.login_url, "/srun_portal_pc_success.php");
        let payload = url::form_urlencoded::Serializer::new(String::new())
            .append_pair(
                "action",
                if form.action.trim().is_empty() {
                    "auto_logout"
                } else {
                    form.action.as_str()
                },
            )
            .append_pair("ac_id", &form.ac_id)
            .append_pair("info", &form.info)
            .append_pair("user_ip", &form.user_ip)
            .append_pair("username", username)
            .finish();
        let response = self
            .transport
            .request(
                "POST",
                &post_url,
                self.build_form_headers(&page_data.login_url),
                payload,
                page_data.cookies.clone(),
                1,
            )
            .await?;
        Ok(response.text.trim().to_string())
    }

    fn build_form_headers(&self, referer_url: &str) -> HashMap<String, String> {
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
}

fn is_legacy_success_page(html: &str) -> bool {
    parse_legacy_portal_success_page(html).is_ok() || html.contains("auto_logout")
}

fn is_portal_login_success(response_text: &str) -> bool {
    response_text.starts_with("login_ok,")
        || response_text.contains("Authentication success")
        || response_text.contains("Portal not response")
}

fn parse_success_logout_form(html: &str) -> SuccessLogoutForm {
    SuccessLogoutForm {
        action: extract_input_value(html, "action"),
        ac_id: extract_input_value(html, "ac_id"),
        info: extract_input_value(html, "info"),
        user_ip: extract_input_value(html, "user_ip"),
        username: extract_input_value(html, "username"),
    }
}

fn extract_input_value(html: &str, name: &str) -> String {
    let pattern = format!(
        r#"(?is)<input[^>]+name=["']{}["'][^>]*>"#,
        regex::escape(name)
    );
    let Ok(input_re) = regex::Regex::new(&pattern) else {
        return String::new();
    };
    let Some(input) = input_re.find(html).map(|item| item.as_str()) else {
        return String::new();
    };
    regex::Regex::new(r#"(?is)value=["']([^"']*)["']"#)
        .ok()
        .and_then(|value_re| value_re.captures(input))
        .and_then(|captures| captures.get(1))
        .map(|value| decode_basic_html_entities(value.as_str()))
        .unwrap_or_default()
}

fn decode_basic_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
