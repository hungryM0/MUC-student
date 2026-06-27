use crate::application::error::{AppError, AppResult};
use crate::domain::models::traffic::OnlineDeviceRecord;
use crate::infrastructure::parsers::online_device_parser::parse_online_devices;
use crate::infrastructure::parsers::portal_page_parser::extract_meta_content;

const FREE_PRODUCT_QUOTA_GB: f64 = 70.0;

pub struct PanelHomeSnapshot {
    pub package_name: String,
    pub billing_policy: String,
    pub used_traffic: String,
    pub product_balance: String,
    pub online_devices: Vec<OnlineDeviceRecord>,
    pub matched_local_ip_device: Option<OnlineDeviceRecord>,
}

pub fn parse_home_table(html: &str) -> AppResult<(String, String, String, String)> {
    let row_regex = regex::Regex::new(r"(?is)<tr[^>]*>(?P<body>.*?)</tr>").expect("valid regex");
    let cell_regex =
        regex::Regex::new(r"(?is)<t[dh][^>]*>(?P<cell>.*?)</t[dh]>").expect("valid regex");
    let strip_regex = regex::Regex::new(r"(?is)<[^>]+>").expect("valid regex");

    let mut rows: Vec<Vec<String>> = Vec::new();
    for row_caps in row_regex.captures_iter(html) {
        let body = row_caps
            .name("body")
            .map(|m| m.as_str())
            .unwrap_or_default();
        let cells = cell_regex
            .captures_iter(body)
            .filter_map(|caps| caps.name("cell"))
            .map(|cell| {
                strip_regex
                    .replace_all(cell.as_str(), " ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>();
        if !cells.is_empty() {
            rows.push(cells);
        }
    }

    for (idx, row) in rows.iter().enumerate() {
        let headers = ["产品名称", "计费策略", "已用流量", "产品余额"];
        if headers
            .iter()
            .all(|name| row.iter().any(|cell| cell == name))
        {
            let Some(package_idx) = row.iter().position(|cell| cell == "产品名称") else {
                continue;
            };
            let Some(billing_idx) = row.iter().position(|cell| cell == "计费策略") else {
                continue;
            };
            let Some(used_idx) = row.iter().position(|cell| cell == "已用流量") else {
                continue;
            };
            let Some(balance_idx) = row.iter().position(|cell| cell == "产品余额") else {
                continue;
            };
            for data_row in rows.iter().skip(idx + 1) {
                let Some(max_idx) = [package_idx, billing_idx, used_idx, balance_idx]
                    .iter()
                    .max()
                    .copied()
                else {
                    continue;
                };
                if data_row.len() <= max_idx {
                    continue;
                }
                return Ok((
                    non_empty(&data_row[package_idx], "未知套餐"),
                    non_empty(&data_row[billing_idx], "-"),
                    non_empty(&data_row[used_idx], "-"),
                    non_empty(&data_row[balance_idx], "-"),
                ));
            }
        }
    }

    Err(AppError::Network(
        "登录成功了，但没在 /home 里找到流量表格".to_string(),
    ))
}

pub fn build_product_balance_texts(html: &str) -> (String, String) {
    let re = regex::Regex::new(r"可用流量[:：]\s*([0-9]+(?:\.[0-9]+)?)\s*([KMGT])B?")
        .expect("valid regex");
    let mut package_total_gb = 0.0;
    for caps in re.captures_iter(html) {
        let value = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
        let unit = caps
            .get(2)
            .map(|m| m.as_str().to_uppercase())
            .unwrap_or_default();
        package_total_gb += convert_to_gigabytes(value, &unit);
    }
    let total_gb = FREE_PRODUCT_QUOTA_GB + package_total_gb;
    let included = if package_total_gb > 0.0 {
        format!("含{package_total_gb:.2}GB套餐流量")
    } else {
        String::new()
    };
    (format!("{total_gb:.2}GB"), included)
}

pub fn extract_csrf_meta(html: &str) -> (String, String) {
    (
        extract_meta_content(html, "csrf-param"),
        extract_meta_content(html, "csrf-token"),
    )
}

pub fn match_local_ip_device(
    online_devices: &[OnlineDeviceRecord],
    local_ip: &str,
) -> Option<OnlineDeviceRecord> {
    let local_ip = local_ip.trim();
    if local_ip.is_empty() || local_ip == "unknown" {
        return None;
    }
    online_devices
        .iter()
        .find(|record| record.ip.trim() == local_ip)
        .cloned()
}

pub fn parse_panel_home(html: &str, local_ip: Option<&str>) -> AppResult<PanelHomeSnapshot> {
    let (package_name, billing_policy, used_traffic, _) = parse_home_table(html)?;
    let (product_balance, _included_package_text) = build_product_balance_texts(html);
    let online_devices = parse_online_devices(html);
    let matched = match_local_ip_device(&online_devices, local_ip.unwrap_or_default());
    Ok(PanelHomeSnapshot {
        package_name,
        billing_policy,
        used_traffic,
        product_balance,
        online_devices,
        matched_local_ip_device: matched,
    })
}

fn convert_to_gigabytes(value: f64, unit: &str) -> f64 {
    match unit {
        "K" => value / 1024.0 / 1024.0,
        "M" => value / 1024.0,
        "G" => value,
        "T" => value * 1024.0,
        _ => 0.0,
    }
}

fn non_empty(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_product_balance_texts, extract_csrf_meta, match_local_ip_device, parse_home_table,
        parse_panel_home,
    };
    use crate::domain::models::traffic::OnlineDeviceRecord;

    #[test]
    fn parses_home_table_and_combines_package_traffic() {
        let html = r#"
        <table>
          <tr>
            <th>产品名称</th><th>计费策略</th><th>已用流量</th><th>产品余额</th>
          </tr>
          <tr>
            <td>校园网</td><td>免费70GB + 套餐</td><td>12.50GB</td><td>57.50GB</td>
          </tr>
        </table>
        <div>可用流量：10GB</div>
        <div>可用流量：512MB</div>
        "#;

        let (package_name, billing_policy, used_traffic, product_balance) =
            parse_home_table(html).expect("parse table");
        let (total, included) = build_product_balance_texts(html);

        assert_eq!(package_name, "校园网");
        assert_eq!(billing_policy, "免费70GB + 套餐");
        assert_eq!(used_traffic, "12.50GB");
        assert_eq!(product_balance, "57.50GB");
        assert_eq!(total, "80.50GB");
        assert_eq!(included, "含10.50GB套餐流量");
    }

    #[test]
    fn parses_panel_home_with_matching_local_device() {
        let snapshot = parse_panel_home(
            r#"
            <meta name="csrf-param" content="_csrf">
            <meta name="csrf-token" content="token-1">
            <table>
              <tr><th>产品名称</th><th>计费策略</th><th>已用流量</th><th>产品余额</th></tr>
              <tr><td>校园网</td><td>免费70GB</td><td>1.00GB</td><td>69.00GB</td></tr>
            </table>
            <tr data-key="device-a">
              <td data-col-seq="1">10.151.119.57</td>
              <td><a href="/home/delete?id=device-a">下线</a></td>
            </tr>
            "#,
            Some("10.151.119.57"),
        )
        .expect("parse panel home");

        assert_eq!(snapshot.package_name, "校园网");
        assert_eq!(snapshot.billing_policy, "免费70GB");
        assert_eq!(snapshot.used_traffic, "1.00GB");
        assert_eq!(snapshot.product_balance, "70.00GB");
        assert_eq!(snapshot.online_devices.len(), 1);
        assert_eq!(
            snapshot
                .matched_local_ip_device
                .as_ref()
                .map(|item| item.device_id.as_str()),
            Some("device-a")
        );
    }

    #[test]
    fn extracts_csrf_meta_and_ignores_unknown_local_ip() {
        let html = r#"<meta name="csrf-param" content="_csrf"><meta name="csrf-token" content=" token-1 ">"#;
        let devices = vec![OnlineDeviceRecord {
            ip: "10.151.119.57".to_string(),
            device_id: "device-a".to_string(),
            logout_path: "/home/delete?id=device-a".to_string(),
        }];

        assert_eq!(
            extract_csrf_meta(html),
            ("_csrf".to_string(), "token-1".to_string())
        );
        assert!(match_local_ip_device(&devices, "unknown").is_none());
        assert!(match_local_ip_device(&devices, "").is_none());
    }
}
