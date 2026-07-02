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

    let response = client
        .get(UPDATE_FEED_URL)
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
