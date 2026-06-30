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
