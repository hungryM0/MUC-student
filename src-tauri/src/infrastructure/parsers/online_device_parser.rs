use crate::domain::models::traffic::OnlineDeviceRecord;

pub fn clean_html_text(raw_html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").expect("valid regex");
    re.replace_all(raw_html, " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_online_devices(html: &str) -> Vec<OnlineDeviceRecord> {
    let row_pattern = regex::Regex::new(
        r#"(?is)<tr[^>]*data-key=["'](?P<device_id>[^"']+)["'][^>]*>(?P<body>.*?)</tr>"#,
    )
    .expect("valid regex");
    let cell_pattern =
        regex::Regex::new(r#"(?is)<td[^>]*data-col-seq=["']1["'][^>]*>(?P<cell>.*?)</td>"#)
            .expect("valid regex");
    let href_pattern =
        regex::Regex::new(r#"(?is)<a[^>]*href=["'](?P<href>[^"']*?/home/delete[^"']*)["']"#)
            .expect("valid regex");

    let mut records = Vec::new();
    for captures in row_pattern.captures_iter(html) {
        let row_html = captures.get(0).map(|m| m.as_str()).unwrap_or_default();
        if !row_html.to_lowercase().contains("/home/delete") {
            continue;
        }
        let body = captures
            .name("body")
            .map(|m| m.as_str())
            .unwrap_or_default();
        let ip = cell_pattern
            .captures(body)
            .and_then(|caps| caps.name("cell"))
            .map(|m| clean_html_text(m.as_str()))
            .unwrap_or_default();
        let raw_href = href_pattern
            .captures(row_html)
            .and_then(|caps| caps.name("href"))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let logout_path = normalize_logout_path(&raw_href);
        let device_id = captures
            .name("device_id")
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        if !ip.is_empty() && !device_id.is_empty() && !logout_path.is_empty() {
            records.push(OnlineDeviceRecord {
                ip,
                device_id,
                logout_path,
            });
        }
    }
    records
}

pub fn normalize_logout_path(raw_href: &str) -> String {
    if raw_href.trim().is_empty() {
        return String::new();
    }
    if let Ok(url) = url::Url::parse(raw_href) {
        let mut path = url.path().to_string();
        if let Some(query) = url.query() {
            path.push('?');
            path.push_str(query);
        }
        return path;
    }
    if raw_href.starts_with('/') {
        raw_href.to_string()
    } else {
        format!("/{}", raw_href.trim_start_matches('/'))
    }
}
