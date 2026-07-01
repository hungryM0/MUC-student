use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Local};
use rusqlite::params;
use uuid::Uuid;

use crate::application::error::{AppError, AppResult};
use crate::domain::models::{AccountStore, CachedTrafficSnapshot, PortalAccount};
use crate::domain::policies::traffic_math::normalize_included_package_text;
use crate::infrastructure::persistence::database::AppDatabase;
use crate::infrastructure::security::credential_vault::CredentialVault;

#[derive(Clone)]
pub struct AccountRepository {
    db: AppDatabase,
    vault: Arc<dyn CredentialVault>,
}

#[derive(Clone, Debug)]
pub struct AccountWithPassword {
    pub account: PortalAccount,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct AccountImportRecord {
    pub remark_name: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountImportStats {
    pub imported_count: usize,
    pub overwritten_count: usize,
}

impl AccountRepository {
    pub fn new(db: AppDatabase, vault: Arc<dyn CredentialVault>) -> Self {
        Self { db, vault }
    }

    pub fn vault(&self) -> &Arc<dyn CredentialVault> {
        &self.vault
    }

    pub fn load_store(&self) -> AppResult<AccountStore> {
        let conn = self.db.lock()?;
        let accounts = load_accounts(&conn)?;
        let valid_ids: HashSet<String> =
            accounts.iter().map(|account| account.id.clone()).collect();
        let (mut selected_account_id, mut current_online_account_id) = conn.query_row(
            "SELECT selected_account_id, current_online_account_id FROM selection_state WHERE id = 1",
            [],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;
        if !selected_account_id.is_empty() && !valid_ids.contains(&selected_account_id) {
            selected_account_id.clear();
        }
        if selected_account_id.is_empty() {
            selected_account_id = accounts
                .first()
                .map(|account| account.id.clone())
                .unwrap_or_default();
        }
        if !valid_ids.contains(&current_online_account_id) {
            current_online_account_id.clear();
        }
        let store = AccountStore {
            selected_account_id,
            accounts,
            current_online_account_id,
            status_card_order_snapshot: load_status_card_order(&conn)?,
            cached_traffic_snapshots: load_snapshots(&conn)?,
        };
        Ok(self.normalize_store(store))
    }

    pub fn save_store(&self, store: &AccountStore) -> AppResult<()> {
        let normalized = self.normalize_store(store.clone());
        let mut conn = self.db.lock()?;
        let tx = conn.transaction()?;
        for (index, account) in normalized.accounts.iter().enumerate() {
            tx.execute(
                r#"
                INSERT INTO accounts (id, remark_name, username, sort_order)
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(id) DO UPDATE SET
                    remark_name = excluded.remark_name,
                    username = excluded.username,
                    sort_order = excluded.sort_order
                "#,
                params![
                    account.id,
                    account.remark_name,
                    account.username,
                    index as i64
                ],
            )?;
        }
        let valid_ids: HashSet<String> = normalized
            .accounts
            .iter()
            .map(|account| account.id.clone())
            .collect();
        let existing_ids = load_account_ids(&tx)?;
        for account_id in existing_ids {
            if !valid_ids.contains(&account_id) {
                tx.execute("DELETE FROM accounts WHERE id = ?1", params![account_id])?;
            }
        }
        tx.execute(
            "UPDATE selection_state SET selected_account_id = ?1, current_online_account_id = ?2 WHERE id = 1",
            params![normalized.selected_account_id, normalized.current_online_account_id],
        )?;
        save_status_card_order_tx(&tx, &normalized.status_card_order_snapshot)?;
        save_snapshots_tx(&tx, &normalized.cached_traffic_snapshots)?;
        tx.commit()?;
        Ok(())
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

    pub fn load_accounts_with_passwords(&self) -> AppResult<Vec<AccountWithPassword>> {
        self.load_store()?
            .accounts
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
        if let Err(err) = self.save_store(&store) {
            let _ = self.vault.delete_password(&account.id);
            return Err(err);
        }
        Ok(account)
    }

    pub fn import_accounts(
        &self,
        records: Vec<AccountImportRecord>,
    ) -> AppResult<AccountImportStats> {
        if records.is_empty() {
            return Err(AppError::Validation("号池里没有账号".to_string()));
        }

        let mut seen_usernames = HashSet::new();
        let records = records
            .into_iter()
            .map(|record| {
                let remark_name = Self::require_text(&record.remark_name, "备注名")?;
                let username = Self::require_text(&record.username, "账号")?;
                let password = Self::require_text(&record.password, "密码")?;
                if !seen_usernames.insert(username.clone()) {
                    return Err(AppError::Validation(format!(
                        "号池里有重复账号：{username}"
                    )));
                }
                Ok(AccountImportRecord {
                    remark_name,
                    username,
                    password,
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        let mut store = self.load_store()?;
        let mut stats = AccountImportStats::default();
        let mut created_ids = Vec::new();
        let mut overwritten_passwords = Vec::new();

        for record in records {
            if let Some(existing) = store
                .accounts
                .iter_mut()
                .find(|account| account.username == record.username)
            {
                let old_password = self.vault.get_password(&existing.id)?;
                overwritten_passwords.push((existing.id.clone(), old_password));
                existing.remark_name = record.remark_name;
                self.vault.set_password(&existing.id, &record.password)?;
                stats.overwritten_count += 1;
                continue;
            }

            let account = PortalAccount {
                id: Uuid::new_v4().simple().to_string(),
                remark_name: record.remark_name,
                username: record.username,
            };
            self.vault.set_password(&account.id, &record.password)?;
            created_ids.push(account.id.clone());
            store.accounts.push(account);
            stats.imported_count += 1;
        }

        if let Err(err) = self.save_store(&store) {
            for account_id in created_ids {
                let _ = self.vault.delete_password(&account_id);
            }
            for (account_id, password) in overwritten_passwords {
                let _ = self.vault.set_password(&account_id, &password);
            }
            return Err(err);
        }

        Ok(stats)
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

fn load_accounts(conn: &rusqlite::Connection) -> AppResult<Vec<PortalAccount>> {
    let mut stmt =
        conn.prepare("SELECT id, remark_name, username FROM accounts ORDER BY sort_order, rowid")?;
    let rows = stmt.query_map([], |row| {
        Ok(PortalAccount {
            id: row.get(0)?,
            remark_name: row.get(1)?,
            username: row.get(2)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn load_account_ids(conn: &rusqlite::Connection) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM accounts")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn load_status_card_order(conn: &rusqlite::Connection) -> AppResult<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT account_id FROM status_card_order ORDER BY sort_order, rowid")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn save_status_card_order_tx(tx: &rusqlite::Transaction<'_>, order: &[String]) -> AppResult<()> {
    tx.execute("DELETE FROM status_card_order", [])?;
    for (index, account_id) in order.iter().enumerate() {
        tx.execute(
            "INSERT INTO status_card_order (account_id, sort_order) VALUES (?1, ?2)",
            params![account_id, index as i64],
        )?;
    }
    Ok(())
}

fn load_snapshots(
    conn: &rusqlite::Connection,
) -> AppResult<BTreeMap<String, CachedTrafficSnapshot>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            account_id,
            used_traffic_text,
            product_balance_text,
            included_package_text,
            package_total_text,
            package_available_text,
            online_device_count_text,
            package_text,
            status_text,
            detail_text,
            queried_at,
            progress_percent
        FROM traffic_snapshots
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        let queried_at: Option<String> = row.get(10)?;
        let queried_at = queried_at
            .as_deref()
            .map(DateTime::parse_from_rfc3339)
            .transpose()
            .map(|value| value.map(|dt| dt.with_timezone(&Local)))
            .map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    10,
                    rusqlite::types::Type::Text,
                    Box::new(err),
                )
            })?;
        Ok((
            row.get::<_, String>(0)?,
            CachedTrafficSnapshot {
                used_traffic_text: row.get(1)?,
                product_balance_text: row.get(2)?,
                included_package_text: normalize_included_package_text(&row.get::<_, String>(3)?),
                package_total_text: row.get(4).unwrap_or_default(),
                package_available_text: row.get(5).unwrap_or_default(),
                online_device_count_text: row.get(6)?,
                package_text: row.get(7)?,
                status_text: row.get(8)?,
                detail_text: row.get(9)?,
                queried_at,
                progress_percent: row.get(11)?,
            },
        ))
    })?;
    rows.collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(Into::into)
}

fn save_snapshots_tx(
    tx: &rusqlite::Transaction<'_>,
    snapshots: &BTreeMap<String, CachedTrafficSnapshot>,
) -> AppResult<()> {
    tx.execute("DELETE FROM traffic_snapshots", [])?;
    for (account_id, snapshot) in snapshots {
        tx.execute(
            r#"
            INSERT INTO traffic_snapshots (
                account_id,
                used_traffic_text,
                product_balance_text,
                included_package_text,
                package_total_text,
                package_available_text,
                online_device_count_text,
                package_text,
                status_text,
                detail_text,
                queried_at,
                progress_percent
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                account_id,
                snapshot.used_traffic_text,
                snapshot.product_balance_text,
                snapshot.included_package_text,
                snapshot.package_total_text,
                snapshot.package_available_text,
                snapshot.online_device_count_text,
                snapshot.package_text,
                snapshot.status_text,
                snapshot.detail_text,
                snapshot.queried_at.map(|value| value.to_rfc3339()),
                snapshot.progress_percent,
            ],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use tempfile::tempdir;

    use super::{AccountImportRecord, AccountRepository};
    use crate::domain::models::CachedTrafficSnapshot;
    use crate::infrastructure::persistence::database::AppDatabase;
    use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
    use crate::infrastructure::security::credential_vault::{
        CredentialVault, MemoryCredentialVault,
    };

    #[test]
    fn stores_accounts_without_plaintext_passwords() {
        let root = tempdir().expect("create temp dir");
        let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
        let db = AppDatabase::open(&paths).expect("open db");
        let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
        let repo = AccountRepository::new(db, vault);

        let account = repo
            .add_account("主号", "20260001", "secret-1")
            .expect("add account");
        let store = repo.load_store().expect("load store");

        assert_eq!(store.accounts, vec![account.clone()]);
        assert_eq!(store.selected_account_id, account.id);
        assert_eq!(
            repo.load_account_with_password(&account)
                .expect("load credential")
                .password,
            "secret-1"
        );
    }

    #[test]
    fn saves_cached_snapshots_and_status_order() {
        let root = tempdir().expect("create temp dir");
        let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
        let db = AppDatabase::open(&paths).expect("open db");
        let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
        let repo = AccountRepository::new(db, vault);
        let account = repo
            .add_account("主号", "20260001", "secret-1")
            .expect("add account");

        let mut snapshots = BTreeMap::new();
        snapshots.insert(
            account.id.clone(),
            CachedTrafficSnapshot {
                used_traffic_text: "1G".to_string(),
                status_text: "已同步".to_string(),
                ..Default::default()
            },
        );
        repo.save_cached_traffic_snapshots(snapshots, account.id.clone(), vec![account.id.clone()])
            .expect("save snapshots");

        let store = repo.load_store().expect("load store");
        assert_eq!(
            store
                .cached_traffic_snapshots
                .get(&account.id)
                .expect("snapshot")
                .used_traffic_text,
            "1G"
        );
        assert_eq!(store.status_card_order_snapshot, vec![account.id]);
    }

    #[test]
    fn imports_accounts_and_overwrites_existing_usernames() {
        let root = tempdir().expect("create temp dir");
        let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
        let db = AppDatabase::open(&paths).expect("open db");
        let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
        let repo = AccountRepository::new(db, vault);
        let existing = repo
            .add_account("旧号", "20260001", "old-secret")
            .expect("add account");

        let stats = repo
            .import_accounts(vec![
                AccountImportRecord {
                    remark_name: "重复号".to_string(),
                    username: "20260001".to_string(),
                    password: "new-secret".to_string(),
                },
                AccountImportRecord {
                    remark_name: "新号".to_string(),
                    username: "20260002".to_string(),
                    password: "secret-2".to_string(),
                },
            ])
            .expect("import accounts");

        assert_eq!(stats.imported_count, 1);
        assert_eq!(stats.overwritten_count, 1);
        let store = repo.load_store().expect("load store");
        assert_eq!(store.accounts.len(), 2);
        assert_eq!(store.accounts[0].remark_name, "重复号");
        assert_eq!(
            repo.load_account_with_password(&existing)
                .expect("load existing")
                .password,
            "new-secret"
        );
    }

    #[test]
    fn import_can_overwrite_existing_passwords() {
        let root = tempdir().expect("create temp dir");
        let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
        let db = AppDatabase::open(&paths).expect("open db");
        let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
        let repo = AccountRepository::new(db, vault);
        let existing = repo
            .add_account("旧号", "20260001", "old-secret")
            .expect("add account");

        let stats = repo
            .import_accounts(vec![AccountImportRecord {
                remark_name: "新备注".to_string(),
                username: "20260001".to_string(),
                password: "new-secret".to_string(),
            }])
            .expect("import accounts");

        assert_eq!(stats.imported_count, 0);
        assert_eq!(stats.overwritten_count, 1);
        let store = repo.load_store().expect("load store");
        assert_eq!(store.accounts.len(), 1);
        assert_eq!(store.accounts[0].remark_name, "新备注");
        assert_eq!(
            repo.load_account_with_password(&existing)
                .expect("load existing")
                .password,
            "new-secret"
        );
    }
}
