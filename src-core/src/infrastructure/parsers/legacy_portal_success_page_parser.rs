use crate::application::error::{AppError, AppResult};
use crate::domain::policies::traffic_math::extract_paid_package_quota_from_billing_policy;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyPortalSuccessInfo {
    pub ip: String,
    pub username: String,
    pub used_traffic: String,
    pub billing_policy: String,
    pub paid_package_quota: Option<String>,
}

pub fn parse_legacy_portal_success_page(raw: &str) -> AppResult<LegacyPortalSuccessInfo> {
    let clean = raw.trim().trim_matches('\u{feff}').trim();
    if clean.is_empty() {
        return Err(AppError::Network("旧门户成功页为空".to_string()));
    }
    if clean.contains("not_online") || clean.contains("not online") {
        return Err(AppError::NotFound("当前 IP 未在线".to_string()));
    }

    let text = html_to_text(clean);
    let ip = extract_value(&text, &["当前的ip", "当前IP", "IP地址", "ip"])
        .or_else(|| extract_ip(&text))
        .ok_or_else(|| AppError::Network("旧门户成功页缺少当前 IP".to_string()))?;
    let username = extract_value(&text, &["上网用户", "用户账号", "账号", "用户名"])
        .ok_or_else(|| AppError::Network("旧门户成功页缺少上网用户".to_string()))?;
    let used_traffic = extract_value(&text, &["已用流量", "使用流量"])
        .ok_or_else(|| AppError::Network("旧门户成功页缺少已用流量".to_string()))?;
    let billing_policy =
        extract_value(&text, &["计费方式", "计费策略"]).unwrap_or_else(|| "-".to_string());
    let paid_package_quota = extract_paid_package_quota_from_billing_policy(&billing_policy);

    Ok(LegacyPortalSuccessInfo {
        ip,
        username,
        used_traffic,
        billing_policy,
        paid_package_quota,
    })
}

fn html_to_text(html: &str) -> String {
    let without_scripts =
        regex::Regex::new(r"(?is)<script[^>]*>.*?</script>|<style[^>]*>.*?</style>")
            .expect("valid regex")
            .replace_all(html, " ");
    let with_separators = regex::Regex::new(r"(?i)</?(?:br|p|div|li|tr|td|th|span|label)[^>]*>")
        .expect("valid regex")
        .replace_all(&without_scripts, "\n");
    let stripped = regex::Regex::new(r"(?is)<[^>]+>")
        .expect("valid regex")
        .replace_all(&with_separators, " ");
    decode_basic_entities(&stripped)
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_value(text: &str, labels: &[&str]) -> Option<String> {
    let lines = text.lines().collect::<Vec<_>>();
    for label in labels {
        for (idx, line) in lines.iter().enumerate() {
            if let Some(value) = extract_value_from_line(line, label) {
                return Some(value);
            }
            if line.trim().eq_ignore_ascii_case(label) {
                if let Some(next) =
                    lines
                        .iter()
                        .skip(idx + 1)
                        .map(|item| item.trim())
                        .find(|item| {
                            !item.is_empty()
                                && !labels.iter().any(|label| item.eq_ignore_ascii_case(label))
                        })
                {
                    return Some(trim_value(next));
                }
            }
        }
    }
    None
}

fn extract_value_from_line(line: &str, label: &str) -> Option<String> {
    let label_pos = line.to_lowercase().find(&label.to_lowercase())?;
    let rest = line[label_pos + label.len()..]
        .trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, ':' | '：' | '=' | '-'))
        .trim();
    if rest.is_empty() {
        return None;
    }
    Some(trim_value(rest))
}

fn extract_ip(text: &str) -> Option<String> {
    regex::Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b")
        .ok()?
        .find(text)
        .map(|item| item.as_str().to_string())
}

fn trim_value(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch| matches!(ch, ':' | '：' | '=' | '-' | ',' | '，' | ';' | '；'))
        .trim()
        .to_string()
}

fn decode_basic_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_table_success_page() {
        let info = parse_legacy_portal_success_page(
            r#"
            <table>
              <tr><td>当前的ip</td><td>10.151.119.57</td></tr>
              <tr><td>上网用户</td><td>13377235977</td></tr>
              <tr><td>已用流量</td><td>7.10G</td></tr>
              <tr><td>计费方式</td><td>包月</td></tr>
            </table>
            "#,
        )
        .unwrap();

        assert_eq!(info.ip, "10.151.119.57");
        assert_eq!(info.username, "13377235977");
        assert_eq!(info.used_traffic, "7.10G");
        assert_eq!(info.billing_policy, "包月");
        assert_eq!(info.paid_package_quota, None);
    }

    #[test]
    fn parses_inline_success_page() {
        let info = parse_legacy_portal_success_page(
            "当前的ip：10.151.119.57\n上网用户：13377235977\n已用流量：7.10G\n计费方式：flow",
        )
        .unwrap();

        assert_eq!(info.ip, "10.151.119.57");
        assert_eq!(info.username, "13377235977");
        assert_eq!(info.used_traffic, "7.10G");
        assert_eq!(info.billing_policy, "flow");
        assert_eq!(info.paid_package_quota, None);
    }

    #[test]
    fn parses_paid_package_quota_from_billing_policy() {
        let info = parse_legacy_portal_success_page(
            r#"
            <ul>
              <li>当前的ip：10.151.109.180</li>
              <li>上网用户：25080004</li>
              <li>已用流量：101,264.17M</li>
              <li>计费方式：Package-use-20元30GB</li>
            </ul>
            "#,
        )
        .unwrap();

        assert_eq!(info.billing_policy, "Package-use-20元30GB");
        assert_eq!(info.paid_package_quota, Some("30.00GB".to_string()));
    }

    #[test]
    fn preserves_unlimited_billing_policy() {
        let info = parse_legacy_portal_success_page(
            r#"
            <ul>
              <li>当前的ip：192.0.2.10</li>
              <li>上网用户：2024000000</li>
              <li>已用流量：79.78G</li>
              <li>计费方式：Package-use-50元不限流量（仅当月有效）</li>
            </ul>
            "#,
        )
        .unwrap();

        assert_eq!(
            info.billing_policy,
            "Package-use-50元不限流量（仅当月有效）"
        );
        assert_eq!(info.paid_package_quota, None);
    }

    #[test]
    fn rejects_missing_required_fields() {
        assert!(parse_legacy_portal_success_page("上网用户：13377235977").is_err());
    }
}
