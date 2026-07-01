use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::{
    AccountStore, AppState, NetworkStatus, PortalAccount, UserPreferences,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    pub id: String,
    pub remark_name: String,
    pub username: String,
    pub snapshot: Option<AccountTrafficSnapshot>,
    pub is_current_online: bool,
    pub can_logout_local_device: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreferenceDto {
    pub minimize_to_tray_on_close: bool,
    pub launch_on_startup: bool,
    pub auto_switch_account_on_traffic_exhausted: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoginStateDto {
    pub running: bool,
    pub last_login_time: Option<DateTime<Local>>,
    pub result_text: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RefreshStateDto {
    pub running: bool,
    pub last_quota_refresh_time: Option<DateTime<Local>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PoolQuotaDto {
    pub used_traffic_text: String,
    pub product_balance_text: String,
    pub included_package_text: String,
    pub progress_percent: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshotDto {
    pub network: NetworkStatus,
    pub accounts: Vec<AccountDto>,
    pub selected_account_id: String,
    pub current_online_account_id: String,
    pub pool_quota: PoolQuotaDto,
    pub login_state: LoginStateDto,
    pub refresh_state: RefreshStateDto,
    pub preferences: PreferenceDto,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountPoolImportResultDto {
    pub snapshot: AppSnapshotDto,
    pub imported_count: usize,
    pub overwritten_count: usize,
}

impl From<&UserPreferences> for PreferenceDto {
    fn from(value: &UserPreferences) -> Self {
        Self {
            minimize_to_tray_on_close: value.minimize_to_tray_on_close,
            launch_on_startup: value.launch_on_startup,
            auto_switch_account_on_traffic_exhausted: value
                .auto_switch_account_on_traffic_exhausted,
        }
    }
}

impl From<&AppState> for LoginStateDto {
    fn from(value: &AppState) -> Self {
        Self {
            running: false,
            last_login_time: value.last_login_time,
            result_text: value.last_login_result.clone(),
            message: value.last_login_message.clone(),
        }
    }
}

impl From<&AppState> for RefreshStateDto {
    fn from(value: &AppState) -> Self {
        Self {
            running: false,
            last_quota_refresh_time: value.last_quota_refresh_time,
        }
    }
}

impl AccountDto {
    pub fn from_store(
        account: &PortalAccount,
        store: &AccountStore,
        snapshot: Option<&AccountTrafficSnapshot>,
    ) -> Self {
        Self {
            id: account.id.clone(),
            remark_name: account.remark_name.clone(),
            username: account.username.clone(),
            snapshot: snapshot.cloned(),
            is_current_online: store.current_online_account_id == account.id,
            can_logout_local_device: snapshot
                .and_then(|item| item.matched_local_ip_device.as_ref())
                .is_some()
                && store.current_online_account_id == account.id,
        }
    }
}
