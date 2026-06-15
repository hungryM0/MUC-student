use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::error::{AppError, AppResult};
use crate::domain::models::{AccountStore, CachedTrafficSnapshot, PortalAccount};
use crate::infrastructure::persistence::file_write::{read_json, write_json_atomic};
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
use crate::infrastructure::security::credential_vault::CredentialVault;

#[derive(Clone)]
pub struct AccountRepository {
    paths: RuntimePaths,
    vault: Arc<dyn CredentialVault>,
}

#[derive(Clone, Debug)]
pub struct AccountWithPassword {
    pub account: PortalAccount,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct AccountStoreFile {
    selected_account_id: String,
    current_online_account_id: String,
    status_card_order_snapshot: Vec<String>,
    accounts: Vec<PortalAccount>,
    cached_traffic_snapshots: BTreeMap<String, CachedTrafficSnapshot>,
}

impl AccountRepository {
    pub fn new(paths: RuntimePaths, vault: Arc<dyn CredentialVault>) -> Self {
        Self { paths, vault }
    }

    pub fn load_store(&self) -> AppResult<AccountStore> {
        let path = self.paths.accounts_path();
        let file: AccountStoreFile = read_json(&path)?;
        Ok(self.normalize_store(AccountStore {
            selected_account_id: file.selected_account_id,
            accounts: file.accounts,
            current_online_account_id: file.current_online_account_id,
            status_card_order_snapshot: file.status_card_order_snapshot,
            cached_traffic_snapshots: file.cached_traffic_snapshots,
        }))
    }

    pub fn save_store(&self, store: &AccountStore) -> AppResult<()> {
        let normalized = self.normalize_store(store.clone());
        let payload = AccountStoreFile {
            selected_account_id: normalized.selected_account_id,
            current_online_account_id: normalized.current_online_account_id,
            status_card_order_snapshot: normalized.status_card_order_snapshot,
            accounts: normalized.accounts,
            cached_traffic_snapshots: normalized.cached_traffic_snapshots,
        };
        write_json_atomic(&self.paths.accounts_path(), &payload)
    }

    pub fn ensure_store(&self) -> AppResult<AccountStore> {
        let store = self.load_store()?;
        self.save_store(&store)?;
        Ok(store)
    }

    pub fn get_selected_account(&self, store: &AccountStore) -> Option<PortalAccount> {
        self.get_account_by_id(store, &store.selected_account_id)
    }

    pub fn get_account_by_id(
        &self,
        store: &AccountStore,
        account_id: &str,
    ) -> Option<PortalAccount> {
        store
            .accounts
            .iter()
            .find(|account| account.id == account_id)
            .cloned()
    }

    pub fn load_account_with_password(
        &self,
        account: &PortalAccount,
    ) -> AppResult<AccountWithPassword> {
        Ok(AccountWithPassword {
            account: account.clone(),
            password: self.vault.get_password(&account.id)?,
        })
    }

    pub fn load_accounts_with_passwords(
        &self,
        accounts: &[PortalAccount],
    ) -> AppResult<Vec<AccountWithPassword>> {
        accounts
            .iter()
            .map(|account| self.load_account_with_password(account))
            .collect()
    }

    pub fn add_account(
        &self,
        remark_name: &str,
        username: &str,
        password: &str,
    ) -> AppResult<PortalAccount> {
        let mut store = self.load_store()?;
        let account = PortalAccount {
            id: Uuid::new_v4().simple().to_string(),
            remark_name: Self::require_text(remark_name, "备注名")?,
            username: Self::require_text(username, "账号")?,
        };
        self.ensure_username_unique(&store.accounts, &account.username, "")?;
        self.vault
            .set_password(&account.id, &Self::require_text(password, "密码")?)?;
        store.accounts.push(account.clone());
        if store.selected_account_id.is_empty() {
            store.selected_account_id = account.id.clone();
        }
        self.save_store(&store)?;
        Ok(account)
    }

    pub fn update_account(
        &self,
        account_id: &str,
        remark_name: &str,
        username: &str,
        password: Option<&str>,
    ) -> AppResult<PortalAccount> {
        let mut store = self.load_store()?;
        let clean_username = Self::require_text(username, "账号")?;
        self.ensure_username_unique(&store.accounts, &clean_username, account_id)?;
        let target = store
            .accounts
            .iter_mut()
            .find(|account| account.id == account_id)
            .ok_or_else(|| AppError::NotFound("找不到要编辑的账号".to_string()))?;
        target.remark_name = Self::require_text(remark_name, "备注名")?;
        target.username = clean_username;
        if let Some(password) = password.filter(|text| !text.trim().is_empty()) {
            self.vault.set_password(account_id, password.trim())?;
        }
        let updated = target.clone();
        self.save_store(&store)?;
        Ok(updated)
    }

    pub fn delete_account(&self, account_id: &str) -> AppResult<PortalAccount> {
        let mut store = self.load_store()?;
        let Some(index) = store
            .accounts
            .iter()
            .position(|account| account.id == account_id)
        else {
            return Err(AppError::NotFound("找不到要删除的账号".to_string()));
        };
        let removed = store.accounts.remove(index);
        if store.selected_account_id == account_id {
            store.selected_account_id = store
                .accounts
                .first()
                .map(|account| account.id.clone())
                .unwrap_or_default();
        }
        if store.current_online_account_id == account_id {
            store.current_online_account_id.clear();
        }
        store.cached_traffic_snapshots.remove(account_id);
        store
            .status_card_order_snapshot
            .retain(|id| id != account_id);
        self.save_store(&store)?;
        self.vault.delete_password(account_id)?;
        Ok(removed)
    }

    pub fn select_account(&self, account_id: &str) -> AppResult<PortalAccount> {
        let mut store = self.load_store()?;
        let account = self
            .get_account_by_id(&store, account_id)
            .ok_or_else(|| AppError::NotFound("找不到要选择的账号".to_string()))?;
        store.selected_account_id = account.id.clone();
        self.save_store(&store)?;
        Ok(account)
    }

    pub fn save_cached_traffic_snapshots(
        &self,
        snapshots: BTreeMap<String, CachedTrafficSnapshot>,
        current_online_account_id: String,
        status_card_order_snapshot: Vec<String>,
    ) -> AppResult<()> {
        let mut store = self.load_store()?;
        let valid_ids: HashSet<String> = store
            .accounts
            .iter()
            .map(|account| account.id.clone())
            .collect();
        store.cached_traffic_snapshots = snapshots
            .into_iter()
            .filter(|(account_id, _)| valid_ids.contains(account_id))
            .collect();
        store.current_online_account_id = if valid_ids.contains(&current_online_account_id) {
            current_online_account_id
        } else {
            String::new()
        };
        store.status_card_order_snapshot =
            Self::normalize_order_snapshot(status_card_order_snapshot, &store.accounts);
        self.save_store(&store)
    }

    pub fn normalize_store(&self, mut store: AccountStore) -> AccountStore {
        let mut seen = HashSet::new();
        store.accounts.retain(|account| {
            seen.insert(account.id.clone())
                && !account.remark_name.trim().is_empty()
                && !account.username.trim().is_empty()
        });
        let valid_ids: HashSet<String> = store
            .accounts
            .iter()
            .map(|account| account.id.clone())
            .collect();
        if !store.selected_account_id.is_empty() && !valid_ids.contains(&store.selected_account_id)
        {
            store.selected_account_id.clear();
        }
        if store.selected_account_id.is_empty() {
            store.selected_account_id = store
                .accounts
                .first()
                .map(|account| account.id.clone())
                .unwrap_or_default();
        }
        if !valid_ids.contains(&store.current_online_account_id) {
            store.current_online_account_id.clear();
        }
        store.status_card_order_snapshot =
            Self::normalize_order_snapshot(store.status_card_order_snapshot, &store.accounts);
        store
            .cached_traffic_snapshots
            .retain(|id, _| valid_ids.contains(id));
        store
    }

    fn normalize_order_snapshot(order: Vec<String>, accounts: &[PortalAccount]) -> Vec<String> {
        let valid_ids: HashSet<String> =
            accounts.iter().map(|account| account.id.clone()).collect();
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        for id in order {
            let clean_id = id.trim().to_string();
            if !clean_id.is_empty()
                && valid_ids.contains(&clean_id)
                && seen.insert(clean_id.clone())
            {
                result.push(clean_id);
            }
        }
        for account in accounts {
            if seen.insert(account.id.clone()) {
                result.push(account.id.clone());
            }
        }
        result
    }

    fn require_text(value: &str, field_name: &str) -> AppResult<String> {
        let clean = value.trim();
        if clean.is_empty() {
            Err(AppError::Validation(format!("{field_name}不能为空")))
        } else {
            Ok(clean.to_string())
        }
    }

    fn ensure_username_unique(
        &self,
        accounts: &[PortalAccount],
        username: &str,
        exclude_account_id: &str,
    ) -> AppResult<()> {
        if accounts
            .iter()
            .any(|account| account.id != exclude_account_id && account.username == username)
        {
            Err(AppError::Validation(
                "这个账号已经存在了，别重复加".to_string(),
            ))
        } else {
            Ok(())
        }
    }
}
