use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortalHiddenFields {
    pub ac_id: String,
    pub user_ip: String,
    pub nas_ip: String,
    pub user_mac: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoginResult {
    pub success: bool,
    pub message: String,
    pub login_url: String,
    pub hidden_fields: PortalHiddenFields,
    pub response_text: String,
    pub checked_at: DateTime<Local>,
    pub already_online: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OnlineDeviceRecord {
    pub ip: String,
    pub device_id: String,
    pub logout_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountTrafficSnapshot {
    pub account_id: String,
    pub used_traffic_text: String,
    pub product_balance_text: String,
    pub included_package_text: String,
    pub online_device_count_text: String,
    pub package_text: String,
    pub status_text: String,
    pub detail_text: String,
    pub queried_at: DateTime<Local>,
    pub online_devices: Vec<OnlineDeviceRecord>,
    pub matched_local_ip_device: Option<OnlineDeviceRecord>,
    pub progress_percent: Option<f64>,
}

impl AccountTrafficSnapshot {
    pub fn loading(account_id: impl Into<String>, queried_at: DateTime<Local>) -> Self {
        Self {
            account_id: account_id.into(),
            used_traffic_text: "-".to_string(),
            product_balance_text: "-".to_string(),
            included_package_text: String::new(),
            online_device_count_text: "-".to_string(),
            package_text: "-".to_string(),
            status_text: "查询中...".to_string(),
            detail_text: "正在刷新这个账号的流量与套餐信息".to_string(),
            queried_at,
            online_devices: Vec::new(),
            matched_local_ip_device: None,
            progress_percent: None,
        }
    }

    pub fn failed(
        account_id: impl Into<String>,
        error: impl Into<String>,
        queried_at: DateTime<Local>,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            used_traffic_text: "-".to_string(),
            product_balance_text: "-".to_string(),
            included_package_text: String::new(),
            online_device_count_text: "-".to_string(),
            package_text: "-".to_string(),
            status_text: "查询失败".to_string(),
            detail_text: error.into(),
            queried_at,
            online_devices: Vec::new(),
            matched_local_ip_device: None,
            progress_percent: None,
        }
    }
}
