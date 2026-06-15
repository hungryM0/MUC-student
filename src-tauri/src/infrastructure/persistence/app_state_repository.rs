use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::application::error::AppResult;
use crate::domain::models::{AppState, UserPreferences};
use crate::infrastructure::persistence::file_write::{read_json, write_json_atomic};
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct AppStateFile {
    last_login_time: Option<DateTime<Local>>,
    last_quota_refresh_time: Option<DateTime<Local>>,
    last_login_result: String,
    last_login_message: String,
    recent_account_ids: Vec<String>,
    minimize_to_tray_on_close: bool,
    launch_on_startup: bool,
    auto_switch_account_on_traffic_exhausted: bool,
    migration_version: u32,
}

#[derive(Clone)]
pub struct AppStateRepository {
    paths: RuntimePaths,
}

impl AppStateRepository {
    pub fn new(paths: RuntimePaths) -> Self {
        Self { paths }
    }

    pub fn load_state(&self) -> AppResult<AppState> {
        let file = self.load_file()?;
        Ok(AppState {
            last_login_time: file.last_login_time,
            last_quota_refresh_time: file.last_quota_refresh_time,
            last_login_result: if file.last_login_result.trim().is_empty() {
                "未执行".to_string()
            } else {
                file.last_login_result
            },
            last_login_message: if file.last_login_message.trim().is_empty() {
                "-".to_string()
            } else {
                file.last_login_message
            },
            recent_account_ids: normalize_recent_account_ids(file.recent_account_ids),
            migration_version: file.migration_version,
        })
    }

    pub fn load_preferences(&self) -> AppResult<UserPreferences> {
        let file = self.load_file()?;
        Ok(UserPreferences {
            minimize_to_tray_on_close: file.minimize_to_tray_on_close,
            launch_on_startup: file.launch_on_startup,
            auto_switch_account_on_traffic_exhausted: file.auto_switch_account_on_traffic_exhausted,
        })
    }

    pub fn save_state(&self, state: &AppState) -> AppResult<()> {
        let mut file = self.load_file()?;
        file.last_login_time = state.last_login_time;
        file.last_quota_refresh_time = state.last_quota_refresh_time;
        file.last_login_result = state.last_login_result.clone();
        file.last_login_message = state.last_login_message.clone();
        file.recent_account_ids = normalize_recent_account_ids(state.recent_account_ids.clone());
        file.migration_version = state.migration_version;
        self.save_file(&file)
    }

    pub fn save_preferences(&self, preferences: &UserPreferences) -> AppResult<()> {
        let mut file = self.load_file()?;
        file.minimize_to_tray_on_close = preferences.minimize_to_tray_on_close;
        file.launch_on_startup = preferences.launch_on_startup;
        file.auto_switch_account_on_traffic_exhausted =
            preferences.auto_switch_account_on_traffic_exhausted;
        self.save_file(&file)
    }

    pub fn mark_account_used(&self, account_id: &str) -> AppResult<Vec<String>> {
        let mut state = self.load_state()?;
        let account_id = account_id.trim();
        if account_id.is_empty() {
            return Ok(state.recent_account_ids);
        }
        state.recent_account_ids.retain(|item| item != account_id);
        state.recent_account_ids.insert(0, account_id.to_string());
        state.recent_account_ids = normalize_recent_account_ids(state.recent_account_ids);
        self.save_state(&state)?;
        Ok(state.recent_account_ids)
    }

    pub fn prune_recent_account_ids(
        &self,
        valid_account_ids: &std::collections::HashSet<String>,
    ) -> AppResult<Vec<String>> {
        let mut state = self.load_state()?;
        state
            .recent_account_ids
            .retain(|account_id| valid_account_ids.contains(account_id));
        self.save_state(&state)?;
        Ok(state.recent_account_ids)
    }

    fn load_file(&self) -> AppResult<AppStateFile> {
        read_json(&self.paths.app_state_path())
    }

    fn save_file(&self, file: &AppStateFile) -> AppResult<()> {
        write_json_atomic(&self.paths.app_state_path(), file)
    }
}

fn normalize_recent_account_ids(raw: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for item in raw {
        let id = item.trim().to_string();
        if !id.is_empty() && seen.insert(id.clone()) {
            result.push(id);
        }
        if result.len() >= 20 {
            break;
        }
    }
    result
}
