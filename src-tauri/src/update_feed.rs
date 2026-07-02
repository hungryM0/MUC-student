use std::time::Duration;

use serde::{Deserialize, Serialize};

const UPDATE_FEED_URL: &str = "https://student.hungrym0.com/latest.json";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFeedDto {
    pub version: Option<String>,
    pub notes: Option<String>,
    pub android: Option<AndroidUpdateFeedDto>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidUpdateFeedDto {
    pub version: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn fetch_android_update_feed() -> Result<UpdateFeedDto, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|err| format!("创建更新检查客户端失败：{err}"))?;

    fetch_update_feed_from_url(&client, UPDATE_FEED_URL).await
}

async fn fetch_update_feed_from_url(
    client: &reqwest::Client,
    url: &str,
) -> Result<UpdateFeedDto, String> {
    let response = client
        .get(url)
        .header(reqwest::header::CACHE_CONTROL, "no-cache")
        .send()
        .await
        .map_err(|err| format!("检查更新失败：{err}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("检查更新失败：{}", status.as_u16()));
    }

    response
        .json::<UpdateFeedDto>()
        .await
        .map_err(|err| format!("解析更新信息失败：{err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn fetches_android_update_feed_with_no_cache_header() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/latest.json"))
            .and(header("cache-control", "no-cache"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "version": "2.0.1",
                "notes": "desktop notes",
                "android": {
                    "version": "2.0.2",
                    "url": "https://example.test/MUC-student.apk",
                    "notes": "android notes"
                }
            })))
            .mount(&server)
            .await;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .expect("client");

        let feed = fetch_update_feed_from_url(&client, &format!("{}/latest.json", server.uri()))
            .await
            .expect("update feed");

        assert_eq!(feed.version.as_deref(), Some("2.0.1"));
        let android = feed.android.expect("android feed");
        assert_eq!(android.version.as_deref(), Some("2.0.2"));
        assert_eq!(
            android.url.as_deref(),
            Some("https://example.test/MUC-student.apk")
        );
        assert_eq!(android.notes.as_deref(), Some("android notes"));
    }

    #[tokio::test]
    async fn maps_http_status_and_invalid_json_to_readable_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing.json"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/broken.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{bad json"))
            .mount(&server)
            .await;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .expect("client");

        let status_error =
            fetch_update_feed_from_url(&client, &format!("{}/missing.json", server.uri()))
                .await
                .expect_err("status error");
        assert_eq!(status_error, "检查更新失败：503");

        let json_error =
            fetch_update_feed_from_url(&client, &format!("{}/broken.json", server.uri()))
                .await
                .expect_err("json error");
        assert!(json_error.starts_with("解析更新信息失败："));
    }
}
