use std::sync::Arc;

use chrono::{DateTime, Local, LocalResult, NaiveDateTime, TimeZone};
use serde::Deserialize;
use uuid::Uuid;

use crate::application::error::{AppError, AppResult};
use crate::domain::models::{AccountStore, AppState, CachedTrafficSnapshot, PortalAccount};
use crate::infrastructure::persistence::account_repository::AccountRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
use crate::infrastructure::security::credential_vault::CredentialVault;

#[derive(Debug, Deserialize)]
struct LegacyAccountRaw {
    #[serde(default)]
    id: Option<String>,
    #[serde(default, alias = "remark_name", alias = "remarkName")]
    remark_name: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct LegacyCachedTrafficSnapshot {
    #[serde(default, alias = "used_traffic_text", alias = "usedTrafficText")]
    used_traffic_text: String,
    #[serde(default, alias = "product_balance_text", alias = "productBalanceText")]
    product_balance_text: String,
    #[serde(
        default,
        alias = "included_package_text",
        alias = "includedPackageText"
    )]
    included_package_text: String,
    #[serde(
        default,
        alias = "online_device_count_text",
        alias = "onlineDeviceCountText"
    )]
    online_device_count_text: String,
    #[serde(default, alias = "package_text", alias = "packageText")]
    package_text: String,
    #[serde(default, alias = "status_text", alias = "statusText")]
    status_text: String,
    #[serde(default, alias = "detail_text", alias = "detailText")]
    detail_text: String,
    #[serde(default, alias = "queried_at", alias = "queriedAt")]
    queried_at: Option<String>,
    #[serde(default, alias = "progress_percent", alias = "progressPercent")]
    progress_percent: Option<f64>,
}

impl TryFrom<LegacyCachedTrafficSnapshot> for CachedTrafficSnapshot {
    type Error = AppError;

    fn try_from(value: LegacyCachedTrafficSnapshot) -> Result<Self, Self::Error> {
        Ok(Self {
            used_traffic_text: value.used_traffic_text,
            product_balance_text: value.product_balance_text,
            included_package_text: value.included_package_text,
            online_device_count_text: value.online_device_count_text,
            package_text: value.package_text,
            status_text: value.status_text,
            detail_text: value.detail_text,
            queried_at: parse_legacy_local_datetime(value.queried_at.as_deref())?,
            progress_percent: value.progress_percent,
        })
    }
}

#[derive(Debug, Deserialize, Default)]
struct LegacyAccountStoreRaw {
    #[serde(default, alias = "selected_account_id", alias = "selectedAccountId")]
    selected_account_id: Option<String>,
    #[serde(
        default,
        alias = "current_online_account_id",
        alias = "currentOnlineAccountId"
    )]
    current_online_account_id: Option<String>,
    #[serde(
        default,
        alias = "status_card_order_snapshot",
        alias = "statusCardOrderSnapshot"
    )]
    status_card_order_snapshot: Option<Vec<String>>,
    accounts: Option<Vec<LegacyAccountRaw>>,
    #[serde(
        default,
        alias = "cached_traffic_snapshots",
        alias = "cachedTrafficSnapshots"
    )]
    cached_traffic_snapshots:
        Option<std::collections::BTreeMap<String, LegacyCachedTrafficSnapshot>>,
}

#[derive(Debug, Deserialize, Default)]
struct LegacyAppStateRaw {
    #[serde(default, alias = "last_login_time", alias = "lastLoginTime")]
    last_login_time: Option<String>,
    #[serde(
        default,
        alias = "last_quota_refresh_time",
        alias = "lastQuotaRefreshTime"
    )]
    last_quota_refresh_time: Option<String>,
    #[serde(default, alias = "last_login_result", alias = "lastLoginResult")]
    last_login_result: Option<String>,
    #[serde(default, alias = "last_login_message", alias = "lastLoginMessage")]
    last_login_message: Option<String>,
    #[serde(default, alias = "recent_account_ids", alias = "recentAccountIds")]
    recent_account_ids: Option<Vec<String>>,
}

pub struct MigrationService {
    paths: RuntimePaths,
    vault: Arc<dyn CredentialVault>,
    account_repo: AccountRepository,
    app_state_repo: AppStateRepository,
}

impl MigrationService {
    pub fn new(
        paths: RuntimePaths,
        vault: Arc<dyn CredentialVault>,
        account_repo: AccountRepository,
        app_state_repo: AppStateRepository,
    ) -> Self {
        Self {
            paths,
            vault,
            account_repo,
            app_state_repo,
        }
    }

    pub fn migrate_if_needed(&self) -> AppResult<bool> {
        if self.paths.accounts_path().exists() || self.paths.app_state_path().exists() {
            return self.import_legacy_if_current_store_empty();
        }
        if !self.paths.legacy_accounts_path().exists()
            && !self.paths.legacy_app_state_path().exists()
        {
            self.account_repo.ensure_store()?;
            let mut state = self.app_state_repo.load_state()?;
            state.migration_version = 1;
            self.app_state_repo.save_state(&state)?;
            return Ok(false);
        }

        self.import_legacy_store(true)
    }

    pub fn import_legacy_if_current_store_empty(&self) -> AppResult<bool> {
        if !self.paths.legacy_accounts_path().exists()
            && !self.paths.legacy_app_state_path().exists()
        {
            return Ok(false);
        }
        if !self.account_repo.load_store()?.accounts.is_empty() {
            return Ok(false);
        }
        self.import_legacy_store(false)
    }

    fn import_legacy_store(&self, remove_legacy_source: bool) -> AppResult<bool> {
        let legacy = self.load_legacy_accounts()?;

        let mut accounts = Vec::new();
        let mut written_credential_ids = Vec::new();
        for raw in legacy.accounts.unwrap_or_default() {
            let id = raw
                .id
                .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
            let remark_name = raw.remark_name.unwrap_or_default().trim().to_string();
            let username = raw.username.unwrap_or_default().trim().to_string();
            let password = raw.password.unwrap_or_default();
            if remark_name.is_empty() || username.is_empty() {
                continue;
            }
            if !password.is_empty() {
                self.vault.set_password(&id, &password)?;
                written_credential_ids.push(id.clone());
            }
            accounts.push(PortalAccount {
                id,
                remark_name,
                username,
            });
        }

        let store = AccountStore {
            selected_account_id: legacy.selected_account_id.unwrap_or_default(),
            accounts,
            current_online_account_id: legacy.current_online_account_id.unwrap_or_default(),
            status_card_order_snapshot: legacy.status_card_order_snapshot.unwrap_or_default(),
            cached_traffic_snapshots: legacy
                .cached_traffic_snapshots
                .unwrap_or_default()
                .into_iter()
                .map(|(account_id, snapshot)| {
                    Ok((account_id, CachedTrafficSnapshot::try_from(snapshot)?))
                })
                .collect::<AppResult<_>>()?,
        };

        if let Err(err) = self.account_repo.save_store(&store) {
            for id in written_credential_ids {
                let _ = self.vault.delete_password(&id);
            }
            return Err(err);
        }

        let mut state = self.app_state_repo.load_state().unwrap_or_default();
        state = self.merge_legacy_app_state(state)?;
        state.migration_version = 1;
        if let Err(err) = self.app_state_repo.save_state(&state) {
            for id in written_credential_ids {
                let _ = self.vault.delete_password(&id);
            }
            let _ = std::fs::remove_file(self.paths.accounts_path());
            return Err(err);
        }

        if remove_legacy_source {
            self.remove_legacy_plaintext_files()?;
        }
        Ok(true)
    }

    fn load_legacy_accounts(&self) -> AppResult<LegacyAccountStoreRaw> {
        let legacy_text =
            std::fs::read_to_string(self.paths.legacy_accounts_path()).unwrap_or_default();
        if legacy_text.trim().is_empty() {
            Ok(LegacyAccountStoreRaw::default())
        } else {
            serde_json::from_str(&legacy_text)
                .map_err(|err| AppError::Storage(format!("旧 accounts.json 格式错误：{err}")))
        }
    }

    fn merge_legacy_app_state(&self, mut state: AppState) -> AppResult<AppState> {
        let legacy_text =
            std::fs::read_to_string(self.paths.legacy_app_state_path()).unwrap_or_default();
        if legacy_text.trim().is_empty() {
            return Ok(state);
        }
        let legacy: LegacyAppStateRaw = serde_json::from_str(&legacy_text)
            .map_err(|err| AppError::Storage(format!("旧 app_state.json 格式错误：{err}")))?;
        state.last_login_time = parse_legacy_local_datetime(legacy.last_login_time.as_deref())?;
        state.last_quota_refresh_time =
            parse_legacy_local_datetime(legacy.last_quota_refresh_time.as_deref())?;
        if let Some(result) = legacy
            .last_login_result
            .filter(|text| !text.trim().is_empty())
        {
            state.last_login_result = result;
        }
        if let Some(message) = legacy
            .last_login_message
            .filter(|text| !text.trim().is_empty())
        {
            state.last_login_message = message;
        }
        if let Some(recent_account_ids) = legacy.recent_account_ids {
            state.recent_account_ids = recent_account_ids;
        }
        Ok(state)
    }

    fn remove_legacy_plaintext_files(&self) -> AppResult<()> {
        for path in [
            self.paths.legacy_accounts_path(),
            self.paths.legacy_app_state_path(),
        ] {
            if path.exists() && path.parent() == Some(self.paths.legacy_root()) {
                std::fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}

fn parse_legacy_local_datetime(input: Option<&str>) -> AppResult<Option<DateTime<Local>>> {
    let Some(text) = input.map(str::trim).filter(|text| !text.is_empty()) else {
        return Ok(None);
    };
    if let Ok(value) = DateTime::parse_from_rfc3339(text) {
        return Ok(Some(value.with_timezone(&Local)));
    }
    let naive = NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f")
        .map_err(|err| AppError::Storage(format!("旧时间格式错误：{err}")))?;
    match Local.from_local_datetime(&naive) {
        LocalResult::Single(value) => Ok(Some(value)),
        LocalResult::Ambiguous(first, _) => Ok(Some(first)),
        LocalResult::None => Err(AppError::Storage(
            "旧时间格式无法映射到本地时区".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::tempdir;

    use super::MigrationService;
    use crate::infrastructure::persistence::account_repository::AccountRepository;
    use crate::infrastructure::persistence::app_state_repository::AppStateRepository;
    use crate::infrastructure::persistence::file_write::write_json_atomic;
    use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
    use crate::infrastructure::security::credential_vault::{
        CredentialVault, MemoryCredentialVault,
    };

    #[test]
    fn backfills_empty_app_data_from_legacy_seed() {
        let app_data = tempdir().expect("create app_data dir");
        let legacy_root = tempdir().expect("create legacy dir");
        let paths = RuntimePaths::new(
            app_data.path().to_path_buf(),
            app_data.path().to_path_buf(),
            legacy_root.path().to_path_buf(),
        )
        .expect("build runtime paths");
        let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
        let account_repo = AccountRepository::new(paths.clone(), vault.clone());
        let app_state_repo = AppStateRepository::new(paths.clone());

        write_json_atomic(
            &paths.accounts_path(),
            &serde_json::json!({
                "selectedAccountId": "",
                "currentOnlineAccountId": "",
                "statusCardOrderSnapshot": [],
                "accounts": [],
                "cachedTrafficSnapshots": {},
            }),
        )
        .expect("write empty current account store");
        write_json_atomic(
            &paths.legacy_accounts_path(),
            &serde_json::json!({
                "selected_account_id": "acc-1",
                "accounts": [
                    {
                        "id": "acc-1",
                        "remark_name": "主号",
                        "username": "20260001",
                        "password": "secret-1"
                    }
                ],
                "cached_traffic_snapshots": {
                    "acc-1": {
                        "used_traffic_text": "12.5G",
                        "product_balance_text": "45.00GB",
                        "status_text": "已同步"
                    }
                }
            }),
        )
        .expect("write legacy accounts");
        write_json_atomic(
            &paths.legacy_app_state_path(),
            &serde_json::json!({
                "last_login_result": "成功",
                "last_login_message": "HTTP 接口登录成功"
            }),
        )
        .expect("write legacy app state");

        let migration = MigrationService::new(
            paths.clone(),
            vault,
            account_repo.clone(),
            app_state_repo.clone(),
        );

        let imported = migration.migrate_if_needed().expect("run migration");
        assert!(imported);

        let store = account_repo.load_store().expect("load imported store");
        assert_eq!(store.accounts.len(), 1);
        assert_eq!(store.accounts[0].remark_name, "主号");
        assert_eq!(
            account_repo
                .load_account_with_password(&store.accounts[0])
                .expect("load imported credential")
                .password,
            "secret-1"
        );
        let state = app_state_repo
            .load_state()
            .expect("load imported app state");
        assert_eq!(state.last_login_result, "成功");
        assert_eq!(state.last_login_message, "HTTP 接口登录成功");
        assert!(paths.legacy_accounts_path().exists());
        assert!(paths.legacy_app_state_path().exists());
    }
}
