use scraper::{Html, Selector};
use url::Url;

use crate::application::error::{AppError, AppResult};
use crate::domain::models::PortalHiddenFields;
use crate::infrastructure::network::models::YiiLoginFormData;

pub fn is_yii_login_page(html: &str) -> bool {
    let lower = html.to_lowercase();
    lower.contains("loginform[username]")
        && lower.contains("loginform[password]")
        && lower.contains("loginform[verifycode]")
}

pub fn is_traffic_home_page(html: &str) -> bool {
    html.contains("产品名称") && html.contains("已用流量") && html.contains("产品余额")
}

pub fn parse_hidden_fields(html: &str, page_url: &str) -> PortalHiddenFields {
    let doc = Html::parse_document(html);
    let selector = Selector::parse("input").expect("valid selector");
    let mut fields = PortalHiddenFields::default();
    for element in doc.select(&selector) {
        let name = element.value().attr("name").unwrap_or_default();
        let value = element
            .value()
            .attr("value")
            .unwrap_or_default()
            .to_string();
        match name {
            "ac_id" => fields.ac_id = value,
            "user_ip" => fields.user_ip = value,
            "nas_ip" => fields.nas_ip = value,
            "user_mac" => fields.user_mac = value,
            _ => {}
        }
    }
    if fields.ac_id.is_empty() {
        if let Ok(url) = Url::parse(page_url) {
            if let Some((_, value)) = url.query_pairs().find(|(key, _)| key == "ac_id") {
                fields.ac_id = value.to_string();
            }
        }
    }
    fields
}

pub fn parse_yii_login_form(html: &str, page_url: &str) -> AppResult<YiiLoginFormData> {
    let doc = Html::parse_document(html);
    let input_selector = Selector::parse("input").expect("valid selector");
    let form_selector = Selector::parse("form").expect("valid selector");
    let img_selector = Selector::parse("img").expect("valid selector");

    let mut csrf_name = String::new();
    let mut csrf_value = String::new();
    for input in doc.select(&input_selector) {
        let name = input.value().attr("name").unwrap_or_default();
        if name.starts_with("_csrf") {
            csrf_name = name.to_string();
            csrf_value = input.value().attr("value").unwrap_or_default().to_string();
            break;
        }
    }
    if csrf_name.is_empty() || csrf_value.is_empty() {
        return Err(AppError::Network(
            "登录页缺少 CSRF 字段，无法提交表单".to_string(),
        ));
    }

    let mut action = "/login".to_string();
    for form in doc.select(&form_selector) {
        let id = form.value().attr("id").unwrap_or_default();
        let candidate = form.value().attr("action").unwrap_or_default();
        if id == "login-form" || candidate.contains("/login") {
            action = if candidate.is_empty() {
                "/login".to_string()
            } else {
                candidate.to_string()
            };
            break;
        }
    }

    let mut captcha = "/site/captcha?refresh=1".to_string();
    for img in doc.select(&img_selector) {
        let id = img.value().attr("id").unwrap_or_default();
        let src = img.value().attr("src").unwrap_or_default();
        if id == "loginform-verifycode-image" || src.contains("captcha") {
            captcha = src.to_string();
            break;
        }
    }

    Ok(YiiLoginFormData {
        csrf_name,
        csrf_value,
        captcha_url: join_url(page_url, &captcha),
        action_url: join_url(page_url, &action),
    })
}

pub fn extract_yii_error_message(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").expect("valid regex");
    let text = re.replace_all(html, " ");
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for key in [
        "验证码不正确",
        "用户名不能为空",
        "密码不能为空",
        "用户名或密码错误",
        "请修复以下错误",
    ] {
        if text.contains(key) {
            return key.to_string();
        }
    }
    if text.is_empty() {
        "登录失败".to_string()
    } else {
        text.chars().take(80).collect()
    }
}

pub fn normalize_captcha_code(raw_code: &str) -> String {
    raw_code
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(4)
        .collect()
}

pub fn extract_meta_content(html: &str, meta_name: &str) -> String {
    let doc = Html::parse_document(html);
    let selector = Selector::parse("meta").expect("valid selector");
    for meta in doc.select(&selector) {
        if meta.value().attr("name") == Some(meta_name) {
            return meta
                .value()
                .attr("content")
                .unwrap_or_default()
                .trim()
                .to_string();
        }
    }
    String::new()
}

pub fn join_url(base_url: &str, path: &str) -> String {
    Url::parse(base_url)
        .and_then(|base| base.join(path))
        .map(|url| url.to_string())
        .unwrap_or_else(|_| path.to_string())
}
