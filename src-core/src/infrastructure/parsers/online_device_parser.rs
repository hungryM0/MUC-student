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

#[cfg(test)]
mod tests {
    use super::{normalize_logout_path, parse_online_devices};

    #[test]
    fn parses_only_rows_with_delete_link() {
        let records = parse_online_devices(
            r#"
            <table>
              <tr data-key="dev-1">
                <td data-col-seq="0">设备</td>
                <td data-col-seq="1"><span>10.151.119.57</span></td>
                <td><a href="/home/delete?id=dev-1">下线</a></td>
              </tr>
              <tr data-key="dev-2">
                <td data-col-seq="1">10.151.119.58</td>
                <td><a href="/home/view?id=dev-2">查看</a></td>
              </tr>
              <tr data-key="">
                <td data-col-seq="1">10.151.119.59</td>
                <td><a href="/home/delete?id=dev-3">下线</a></td>
              </tr>
            </table>
            "#,
        );

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].device_id, "dev-1");
        assert_eq!(records[0].ip, "10.151.119.57");
        assert_eq!(records[0].logout_path, "/home/delete?id=dev-1");
    }

    #[test]
    fn normalizes_absolute_and_relative_logout_paths() {
        assert_eq!(
            normalize_logout_path("http://panel.example/home/delete?id=1&x=2"),
            "/home/delete?id=1&x=2"
        );
        assert_eq!(
            normalize_logout_path("home/delete?id=1"),
            "/home/delete?id=1"
        );
        assert_eq!(
            normalize_logout_path("/home/delete?id=1"),
            "/home/delete?id=1"
        );
        assert_eq!(normalize_logout_path(""), "");
    }
}
