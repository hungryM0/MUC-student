use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    pub is_online: bool,
    pub status_text: String,
    pub ip: String,
    pub checked_at: DateTime<Local>,
}

impl Default for NetworkStatus {
    fn default() -> Self {
        Self {
            is_online: false,
            status_text: "未认证".to_string(),
            ip: "unknown".to_string(),
            checked_at: Local::now(),
        }
    }
}
