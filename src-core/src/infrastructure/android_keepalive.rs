use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::application::dto::AppSnapshotDto;
use crate::application::error::{AppError, AppResult};

const STATE_FILE_NAME: &str = "android_keepalive_state.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AndroidKeepaliveState {
    pub current_account_name: String,
    pub used_traffic_text: String,
    pub product_balance_text: String,
    pub last_updated_at: Option<String>,
}

impl AndroidKeepaliveState {
    pub fn from_snapshot(snapshot: &AppSnapshotDto) -> Self {
        let current_account_name = snapshot
            .accounts
            .iter()
            .find(|account| account.id == snapshot.current_online_account_id)
            .map(|account| account.remark_name.clone())
            .unwrap_or_default();
        Self {
            current_account_name,
            used_traffic_text: snapshot.pool_quota.used_traffic_text.clone(),
            product_balance_text: snapshot.pool_quota.product_balance_text.clone(),
            last_updated_at: snapshot.refresh_state.last_quota_refresh_time.map(|value| {
                value
                    .with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            }),
        }
    }
}

pub fn write_android_keepalive_state(
    app_data_dir: &Path,
    snapshot: &AppSnapshotDto,
) -> AppResult<()> {
    let state_path = android_keepalive_state_path(app_data_dir);
    let payload = serde_json::to_vec_pretty(&AndroidKeepaliveState::from_snapshot(snapshot))
        .map_err(|err| AppError::Storage(format!("序列化 Android 保活状态失败：{err}")))?;
    std::fs::write(state_path, payload)
        .map_err(|err| AppError::Storage(format!("写入 Android 保活状态失败：{err}")))?;
    Ok(())
}

pub fn android_keepalive_state_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(STATE_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::AndroidKeepaliveState;
    use crate::application::dto::{
        AccountDto, AppSnapshotDto, LoginStateDto, PoolQuotaDto, PreferenceDto, RefreshStateDto,
    };
    use crate::domain::models::{traffic::AccountTrafficSnapshot, NetworkStatus};

    #[test]
    fn keepalive_state_uses_pool_quota_instead_of_current_account_snapshot() {
        let current_account_id = "acc-1".to_string();
        let snapshot = AppSnapshotDto {
            network: NetworkStatus::default(),
            accounts: vec![AccountDto {
                id: current_account_id.clone(),
                remark_name: "主号".to_string(),
                username: "20260001".to_string(),
                snapshot: Some(AccountTrafficSnapshot {
                    account_id: current_account_id.clone(),
                    used_traffic_text: "1.00GB".to_string(),
                    product_balance_text: "70.00GB".to_string(),
                    included_package_text: String::new(),
                    package_total_text: String::new(),
                    package_available_text: String::new(),
                    online_device_count_text: "1".to_string(),
                    package_text: "校园网".to_string(),
                    status_text: "已同步".to_string(),
                    detail_text: String::new(),
                    queried_at: Local::now(),
                    online_devices: Vec::new(),
                    matched_local_ip_device: None,
                    progress_percent: Some(1.4),
                }),
                is_current_online: true,
                can_logout_local_device: false,
            }],
            selected_account_id: current_account_id.clone(),
            current_online_account_id: current_account_id,
            pool_quota: PoolQuotaDto {
                used_traffic_text: "3.00GB".to_string(),
                product_balance_text: "140.00GB".to_string(),
                included_package_text: String::new(),
                progress_percent: Some(2.1),
            },
            login_state: LoginStateDto {
                running: false,
                last_login_time: None,
                result_text: "未执行".to_string(),
                message: "-".to_string(),
            },
            refresh_state: RefreshStateDto {
                running: false,
                last_quota_refresh_time: None,
            },
            preferences: PreferenceDto {
                minimize_to_tray_on_close: true,
                launch_on_startup: false,
                auto_switch_account_on_traffic_exhausted: false,
            },
        };

        let state = AndroidKeepaliveState::from_snapshot(&snapshot);

        assert_eq!(state.current_account_name, "主号");
        assert_eq!(state.used_traffic_text, "3.00GB");
        assert_eq!(state.product_balance_text, "140.00GB");
    }
}
