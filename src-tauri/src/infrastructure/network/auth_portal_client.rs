use std::collections::HashMap;

use chrono::Local;

use crate::application::error::{AppError, AppResult};
use crate::domain::models::{LoginResult, PortalHiddenFields};
use crate::infrastructure::network::http_transport::{encode_password, HttpTransport};
use crate::infrastructure::network::models::PortalPageData;
use crate::infrastructure::ocr::OcrProviderChain;
use crate::infrastructure::parsers::portal_page_parser::{
    extract_yii_error_message, is_yii_login_page, join_url, normalize_captcha_code,
    parse_hidden_fields, parse_yii_login_form,
};
use crate::infrastructure::persistence::account_repository::AccountWithPassword;
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct AuthPortalClient {
    settings: AppSettings,
    transport: HttpTransport,
    ocr_chain: std::sync::Arc<OcrProviderChain>,
}

impl AuthPortalClient {
    const RESPONSE_IP_ALREADY_ONLINE: &'static str = "IP has been online, please logout.";

    pub fn new(
        settings: AppSettings,
        transport: HttpTransport,
        ocr_chain: std::sync::Arc<OcrProviderChain>,
    ) -> Self {
        Self {
            settings,
            transport,
            ocr_chain,
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
        })
    }

    pub async fn verify_login(&self, account: &AccountWithPassword) -> AppResult<LoginResult> {
        let page_data = self.fetch_login_page().await?;
        if is_yii_login_page(&page_data.html) {
            return self.verify_login_yii(account).await;
        }

        if page_data.hidden_fields.ac_id.is_empty() {
            return Err(AppError::Network(
                "登录页缺少 ac_id，无法继续验证".to_string(),
            ));
        }

        let response = self.login_with_page_data(account, &page_data).await?;

        let response_text = response.text.trim().to_string();
        let already_online = response_text == Self::RESPONSE_IP_ALREADY_ONLINE;
        let success = response_text.starts_with("login_ok,");
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
            login_url: page_data.login_url,
            hidden_fields: page_data.hidden_fields,
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
        let page_data = self.fetch_login_page().await?;
        if is_yii_login_page(&page_data.html) {
            return self.verify_login_yii(target_account).await;
        }

        if page_data.hidden_fields.ac_id.is_empty() {
            return Err(AppError::Network(
                "登录页缺少 ac_id，无法继续切号".to_string(),
            ));
        }

        self.logout_with_page_data(current_account, &page_data)
            .await?;
        let response = self
            .login_with_page_data(target_account, &page_data)
            .await?;
        let response_text = response.text.trim().to_string();
        let success = response_text.starts_with("login_ok,");

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
            login_url: page_data.login_url,
            hidden_fields: page_data.hidden_fields,
            response_text,
            checked_at: Local::now(),
            already_online: false,
        })
    }

    pub async fn logout_current_ip(&self, account: &AccountWithPassword) -> AppResult<String> {
        let page_data = self.fetch_login_page().await?;
        if is_yii_login_page(&page_data.html) {
            return Err(AppError::Network(
                "当前 Portal 入口需要 OCR 验证码，无法用轻量接口下线".to_string(),
            ));
        }
        if page_data.hidden_fields.ac_id.is_empty() {
            return Err(AppError::Network(
                "登录页缺少 ac_id，无法继续下线".to_string(),
            ));
        }
        self.logout_with_page_data(account, &page_data).await
    }

    async fn verify_login_yii(&self, account: &AccountWithPassword) -> AppResult<LoginResult> {
        let mut last_error = "未知错误".to_string();
        for _ in 0..10 {
            let page_response = self
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
            if !is_yii_login_page(&page_response.text) {
                return Err(AppError::Network("入口页不是可识别的登录表单".to_string()));
            }
            let form = parse_yii_login_form(&page_response.text, &page_response.final_url)?;
            let captcha_response = self
                .transport
                .request(
                    "GET",
                    &form.captcha_url,
                    referer_headers(&page_response.final_url),
                    String::new(),
                    page_response.cookies.clone(),
                    2,
                )
                .await?;
            let captcha_code = normalize_captcha_code(
                &self
                    .ocr_chain
                    .recognize_for_login(&captcha_response.raw_body, form.captcha_sum_hint)
                    .await?,
            );
            if captcha_code.len() < 4 {
                last_error = format!(
                    "OCR 识别结果无效：{}",
                    if captcha_code.is_empty() {
                        "<empty>"
                    } else {
                        &captcha_code
                    }
                );
                continue;
            }
            let payload = url::form_urlencoded::Serializer::new(String::new())
                .append_pair(&form.csrf_name, &form.csrf_value)
                .append_pair("LoginForm[username]", &account.account.username)
                .append_pair("LoginForm[password]", &account.password)
                .append_pair("LoginForm[verifyCode]", &captcha_code)
                .finish();
            let response = self
                .transport
                .request(
                    "POST",
                    &form.action_url,
                    self.build_form_headers(&page_response.final_url),
                    payload,
                    page_response.cookies,
                    3,
                )
                .await?;
            if !is_yii_login_page(&response.text) {
                return Ok(LoginResult {
                    success: true,
                    message: "HTTP 表单登录成功（OCR 验证码）".to_string(),
                    login_url: response.final_url.clone(),
                    hidden_fields: PortalHiddenFields::default(),
                    response_text: response.final_url,
                    checked_at: Local::now(),
                    already_online: false,
                });
            }
            let error_text = extract_yii_error_message(&response.text);
            if error_text.contains("验证码不正确") {
                last_error = format!("验证码识别失败（OCR={captcha_code}）");
                continue;
            }
            if error_text.contains("用户名") || error_text.contains("密码") {
                return Err(AppError::Network(error_text));
            }
            last_error = error_text;
        }
        Err(AppError::Ocr(format!(
            "验证码连续识别失败，已重试 10 次，最后错误：{last_error}"
        )))
    }

    fn build_login_headers(&self, referer_url: &str) -> HashMap<String, String> {
        let mut headers = self.build_form_headers(referer_url);
        headers.insert("X-Requested-With".to_string(), "XMLHttpRequest".to_string());
        headers
    }

    async fn login_with_page_data(
        &self,
        account: &AccountWithPassword,
        page_data: &PortalPageData,
    ) -> AppResult<crate::infrastructure::network::models::HttpResponseData> {
        let post_url = join_url(&page_data.login_url, "/include/auth_action.php");
        let payload = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("action", "login")
            .append_pair("username", &account.account.username)
            .append_pair("password", &encode_password(&account.password))
            .append_pair("ac_id", &page_data.hidden_fields.ac_id)
            .append_pair("user_ip", &page_data.hidden_fields.user_ip)
            .append_pair("nas_ip", &page_data.hidden_fields.nas_ip)
            .append_pair("user_mac", &page_data.hidden_fields.user_mac)
            .append_pair("save_me", "0")
            .append_pair("ajax", "1")
            .finish();
        self.transport
            .request(
                "POST",
                &post_url,
                self.build_login_headers(&page_data.login_url),
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

fn referer_headers(referer_url: &str) -> HashMap<String, String> {
    HashMap::from([("Referer".to_string(), referer_url.to_string())])
}
