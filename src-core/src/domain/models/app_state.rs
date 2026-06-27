use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub last_login_time: Option<DateTime<Local>>,
    pub last_quota_refresh_time: Option<DateTime<Local>>,
    pub last_login_result: String,
    pub last_login_message: String,
    pub recent_account_ids: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_login_time: None,
            last_quota_refresh_time: None,
            last_login_result: "未执行".to_string(),
            last_login_message: "-".to_string(),
            recent_account_ids: Vec::new(),
        }
    }
}
