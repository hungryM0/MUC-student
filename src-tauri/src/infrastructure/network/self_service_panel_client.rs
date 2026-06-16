use std::collections::HashMap;
use std::time::Duration;

use crate::application::error::{AppError, AppResult};
use crate::infrastructure::network::http_transport::HttpTransport;
use crate::infrastructure::network::models::HttpResponseData;
use crate::infrastructure::ocr::OcrProviderChain;
use crate::infrastructure::parsers::online_device_parser::parse_online_devices;
use crate::infrastructure::parsers::panel_home_parser::extract_csrf_meta;
use crate::infrastructure::parsers::portal_page_parser::{
    extract_yii_error_message, is_traffic_home_page, is_yii_login_page, join_url,
    normalize_captcha_code, parse_yii_login_form,
};
use crate::infrastructure::persistence::account_repository::AccountWithPassword;
use crate::infrastructure::persistence::panel_session_repository::PanelSessionRepository;
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct SelfServicePanelClient {
    settings: AppSettings,
    transport: HttpTransport,
    ocr_chain: std::sync::Arc<OcrProviderChain>,
    session_repo: PanelSessionRepository,
}

impl SelfServicePanelClient {
    const LOCAL_DEVICE_VERIFY_RETRY_DELAYS_MS: [u64; 3] = [0, 600, 1200];

    pub fn new(
        settings: AppSettings,
        transport: HttpTransport,
        ocr_chain: std::sync::Arc<OcrProviderChain>,
        session_repo: PanelSessionRepository,
    ) -> Self {
        Self {
            settings,
            transport,
            ocr_chain,
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
        let target_url = if self.settings.traffic_portal_url.trim().is_empty() {
            self.settings.portal_url.clone()
        } else {
            self.settings.traffic_portal_url.clone()
        };
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
                    "GET",
                    &join_url(&target_url, target_path),
                    HashMap::new(),
                    String::new(),
                    saved_cookies,
                    5,
                )
                .await?;
            if is_traffic_home_page(&session_response.text) {
                self.persist_session(account, &session_response.cookies)?;
                return Ok(session_response);
            }
            if !is_yii_login_page(&session_response.text) {
                self.persist_session(account, &session_response.cookies)?;
                return Ok(session_response);
            }
            self.session_repo.clear_session(&account.account.id)?;
        }

        let page_response = self
            .transport
            .request(
                "GET",
                &target_url,
                HashMap::new(),
                String::new(),
                HashMap::new(),
                5,
            )
            .await?;
        if is_traffic_home_page(&page_response.text) {
            self.persist_session(account, &page_response.cookies)?;
            return Ok(page_response);
        }
        if is_yii_login_page(&page_response.text) {
            let mut response = self.login_yii_with_ocr(account, page_response).await?;
            let response_path = url::Url::parse(&response.final_url)
                .ok()
                .map(|url| url.path().trim_end_matches('/').to_string())
                .unwrap_or_default();
            if response_path != target_path.trim_end_matches('/') {
                response = self
                    .transport
                    .request(
                        "GET",
                        &join_url(&response.final_url, target_path),
                        HashMap::new(),
                        String::new(),
                        response.cookies.clone(),
                        5,
                    )
                    .await?;
                if is_yii_login_page(&response.text) {
                    self.session_repo.clear_session(&account.account.id)?;
                    return Err(AppError::Network(
                        "登录态失效，访问目标页面时被重定向回登录页".to_string(),
                    ));
                }
                if !is_traffic_home_page(&response.text) {
                    return Err(AppError::Network(format!(
                        "访问 {target_path} 成功，但页面里没有流量表格"
                    )));
                }
            }
            self.persist_session(account, &response.cookies)?;
            return Ok(response);
        }

        let retry_response = self
            .transport
            .request(
                "GET",
                &join_url(&page_response.final_url, target_path),
                HashMap::new(),
                String::new(),
                page_response.cookies.clone(),
                5,
            )
            .await?;
        if is_traffic_home_page(&retry_response.text) {
            self.persist_session(account, &retry_response.cookies)?;
            return Ok(retry_response);
        }
        if is_yii_login_page(&retry_response.text) {
            return self.login_yii_with_ocr(account, retry_response).await;
        }
        Err(AppError::Network(format!(
            "流量入口不匹配：query_url={}, final_url={}",
            self.settings.traffic_portal_url, page_response.final_url
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
                "POST",
                &join_url(&home_response.final_url, &local_device.logout_path),
                self.build_form_headers(&home_response.final_url),
                payload,
                home_response.cookies.clone(),
                3,
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
                    "GET",
                    &verify_url,
                    HashMap::new(),
                    String::new(),
                    verify_cookies.clone(),
                    5,
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

    async fn login_yii_with_ocr(
        &self,
        account: &AccountWithPassword,
        mut current_page: HttpResponseData,
    ) -> AppResult<HttpResponseData> {
        let mut last_error = "未知错误".to_string();
        for _ in 0..10 {
            if !is_yii_login_page(&current_page.text) {
                return Err(AppError::Network("入口页不是可识别的登录表单".to_string()));
            }
            let form = parse_yii_login_form(&current_page.text, &current_page.final_url)?;
            let captcha_response = self
                .transport
                .request(
                    "GET",
                    &form.captcha_url,
                    referer_headers(&current_page.final_url),
                    String::new(),
                    current_page.cookies.clone(),
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
                current_page = self
                    .transport
                    .request(
                        "GET",
                        &self.settings.traffic_portal_url,
                        HashMap::new(),
                        String::new(),
                        current_page.cookies.clone(),
                        5,
                    )
                    .await?;
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
                    self.build_form_headers(&current_page.final_url),
                    payload,
                    current_page.cookies,
                    3,
                )
                .await?;
            if !is_yii_login_page(&response.text) {
                self.persist_session(account, &response.cookies)?;
                return Ok(response);
            }
            let error_text = extract_yii_error_message(&response.text);
            if error_text.contains("验证码不正确") {
                last_error = format!("验证码识别失败（OCR={captcha_code}）");
                current_page = response;
                continue;
            }
            if error_text.contains("用户名") || error_text.contains("密码") {
                return Err(AppError::Network(error_text));
            }
            last_error = error_text;
            current_page = response;
        }
        Err(AppError::Ocr(format!(
            "验证码连续识别失败，已重试 10 次，最后错误：{last_error}"
        )))
    }

    fn persist_session(
        &self,
        account: &AccountWithPassword,
        cookies: &HashMap<String, String>,
    ) -> AppResult<()> {
        self.session_repo.save_session(&account.account.id, cookies)
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
