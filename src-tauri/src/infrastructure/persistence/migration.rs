use std::sync::Arc;

use serde::Deserialize;
use uuid::Uuid;

use crate::application::error::{AppError, AppResult};
use crate::domain::models::{AccountStore, CachedTrafficSnapshot, PortalAccount};
use crate::infrastructure::persistence::account_repository::AccountRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
use crate::infrastructure::security::credential_vault::CredentialVault;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyAccountRaw {
    id: Option<String>,
    remark_name: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LegacyAccountStoreRaw {
    selected_account_id: Option<String>,
    current_online_account_id: Option<String>,
    status_card_order_snapshot: Option<Vec<String>>,
    accounts: Option<Vec<LegacyAccountRaw>>,
    cached_traffic_snapshots: Option<std::collections::BTreeMap<String, CachedTrafficSnapshot>>,
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
            return Ok(false);
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

        let legacy_text =
            std::fs::read_to_string(self.paths.legacy_accounts_path()).unwrap_or_default();
        let legacy: LegacyAccountStoreRaw = if legacy_text.trim().is_empty() {
            LegacyAccountStoreRaw::default()
        } else {
            serde_json::from_str(&legacy_text)
                .map_err(|err| AppError::Storage(format!("旧 accounts.json 格式错误：{err}")))?
        };

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
            cached_traffic_snapshots: legacy.cached_traffic_snapshots.unwrap_or_default(),
        };

        if let Err(err) = self.account_repo.save_store(&store) {
            for id in written_credential_ids {
                let _ = self.vault.delete_password(&id);
            }
            return Err(err);
        }

        let mut state = self.app_state_repo.load_state().unwrap_or_default();
        state.migration_version = 1;
        if let Err(err) = self.app_state_repo.save_state(&state) {
            for id in written_credential_ids {
                let _ = self.vault.delete_password(&id);
            }
            let _ = std::fs::remove_file(self.paths.accounts_path());
            return Err(err);
        }

        self.remove_legacy_plaintext_files()?;
        Ok(true)
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
