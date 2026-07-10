use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortalAccount {
    pub id: String,
    pub remark_name: String,
    pub username: String,
}

impl PortalAccount {
    pub fn display_name(&self) -> String {
        format!("{}（{}）", self.remark_name, self.username)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CachedTrafficSnapshot {
    pub used_traffic_text: String,
    pub product_balance_text: String,
    pub included_package_text: String,
    pub package_total_text: String,
    pub package_available_text: String,
    pub online_device_count_text: String,
    pub package_text: String,
    pub status_text: String,
    pub detail_text: String,
    #[serde(default)]
    pub is_unlimited_plan: bool,
    pub queried_at: Option<DateTime<Local>>,
    pub progress_percent: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountStore {
    pub selected_account_id: String,
    pub accounts: Vec<PortalAccount>,
    pub current_online_account_id: String,
    pub status_card_order_snapshot: Vec<String>,
    pub cached_traffic_snapshots: std::collections::BTreeMap<String, CachedTrafficSnapshot>,
}
