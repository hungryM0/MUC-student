use scraper::{Html, Selector};
use url::Url;

use crate::domain::models::PortalHiddenFields;

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
