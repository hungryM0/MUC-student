use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Transaction};

use crate::application::error::AppResult;
use crate::domain::models::{AccountStore, CachedTrafficSnapshot, PortalAccount};
use crate::domain::policies::traffic_math::normalize_included_package_text;
use crate::infrastructure::persistence::database::AppDatabase;

#[derive(Clone)]
pub struct AccountSnapshotRepository {
    db: AppDatabase,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AccountSnapshotState {
    pub current_online_account_id: String,
    pub status_card_order_snapshot: Vec<String>,
    pub cached_traffic_snapshots: BTreeMap<String, CachedTrafficSnapshot>,
}

impl AccountSnapshotRepository {
    pub fn new(db: AppDatabase) -> Self {
        Self { db }
    }

    pub fn load_state_for_accounts(
        &self,
        accounts: &[PortalAccount],
    ) -> AppResult<AccountSnapshotState> {
        let conn = self.db.lock()?;
        let state = load_state_from_connection(&conn)?;
        Ok(Self::normalize_state(state, accounts))
    }

    pub fn save_store_state(&self, store: &AccountStore) -> AppResult<()> {
        self.save_state(
            &store.accounts,
            AccountSnapshotState {
                current_online_account_id: store.current_online_account_id.clone(),
                status_card_order_snapshot: store.status_card_order_snapshot.clone(),
                cached_traffic_snapshots: store.cached_traffic_snapshots.clone(),
            },
        )
    }

    pub fn save_cached_traffic_snapshots(
        &self,
        accounts: &[PortalAccount],
        snapshots: BTreeMap<String, CachedTrafficSnapshot>,
        current_online_account_id: String,
        status_card_order_snapshot: Vec<String>,
    ) -> AppResult<()> {
        self.save_state(
            accounts,
            AccountSnapshotState {
                current_online_account_id,
                status_card_order_snapshot,
                cached_traffic_snapshots: snapshots,
            },
        )
    }

    pub fn save_cached_snapshot(
        &self,
        accounts: &[PortalAccount],
        account_id: &str,
        snapshot: CachedTrafficSnapshot,
    ) -> AppResult<()> {
        let mut state = self.load_state_for_accounts(accounts)?;
        state
            .cached_traffic_snapshots
            .insert(account_id.to_string(), snapshot);
        self.save_state(accounts, state)
    }

    pub fn set_current_online_account_id(
        &self,
        accounts: &[PortalAccount],
        current_online_account_id: String,
    ) -> AppResult<String> {
        let mut state = self.load_state_for_accounts(accounts)?;
        state.current_online_account_id = current_online_account_id;
        self.save_state(accounts, state)?;
        let normalized = self.load_state_for_accounts(accounts)?;
        Ok(normalized.current_online_account_id)
    }

    pub fn merge_store(
        &self,
        mut store: AccountStore,
        snapshot_state: AccountSnapshotState,
    ) -> AccountStore {
        store.current_online_account_id = snapshot_state.current_online_account_id;
        store.status_card_order_snapshot = snapshot_state.status_card_order_snapshot;
        store.cached_traffic_snapshots = snapshot_state.cached_traffic_snapshots;
        store
    }

    pub(crate) fn normalize_state(
        mut state: AccountSnapshotState,
        accounts: &[PortalAccount],
    ) -> AccountSnapshotState {
        let valid_ids: HashSet<String> =
            accounts.iter().map(|account| account.id.clone()).collect();
        if !valid_ids.contains(&state.current_online_account_id) {
            state.current_online_account_id.clear();
        }
        state.status_card_order_snapshot =
            Self::normalize_order_snapshot(state.status_card_order_snapshot, accounts);
        state
            .cached_traffic_snapshots
            .retain(|id, _| valid_ids.contains(id));
        state
    }

    fn save_state(&self, accounts: &[PortalAccount], state: AccountSnapshotState) -> AppResult<()> {
        let normalized = Self::normalize_state(state, accounts);
        let mut conn = self.db.lock()?;
        let tx = conn.transaction()?;
        save_state_tx(&tx, &normalized)?;
        tx.commit()?;
        Ok(())
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
}

pub(crate) fn load_state_from_connection(conn: &Connection) -> AppResult<AccountSnapshotState> {
    let current_online_account_id = conn.query_row(
        "SELECT current_online_account_id FROM selection_state WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    Ok(AccountSnapshotState {
        current_online_account_id,
        status_card_order_snapshot: load_status_card_order(conn)?,
        cached_traffic_snapshots: load_snapshots(conn)?,
    })
}

pub(crate) fn save_state_tx(tx: &Transaction<'_>, state: &AccountSnapshotState) -> AppResult<()> {
    tx.execute(
        "UPDATE selection_state SET current_online_account_id = ?1 WHERE id = 1",
        params![state.current_online_account_id],
    )?;
    save_status_card_order_tx(tx, &state.status_card_order_snapshot)?;
    save_snapshots_tx(tx, &state.cached_traffic_snapshots)?;
    Ok(())
}

fn load_status_card_order(conn: &Connection) -> AppResult<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT account_id FROM status_card_order ORDER BY sort_order, rowid")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn save_status_card_order_tx(tx: &Transaction<'_>, order: &[String]) -> AppResult<()> {
    tx.execute("DELETE FROM status_card_order", [])?;
    for (index, account_id) in order.iter().enumerate() {
        tx.execute(
            "INSERT INTO status_card_order (account_id, sort_order) VALUES (?1, ?2)",
            params![account_id, index as i64],
        )?;
    }
    Ok(())
}

fn load_snapshots(conn: &Connection) -> AppResult<BTreeMap<String, CachedTrafficSnapshot>> {
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
            is_unlimited_plan,
            queried_at,
            progress_percent
        FROM traffic_snapshots
        "#,
    )?;
    let rows = stmt.query_map([], |row| {
        let queried_at: Option<String> = row.get(11)?;
        let queried_at = queried_at
            .as_deref()
            .map(DateTime::parse_from_rfc3339)
            .transpose()
            .map(|value| value.map(|dt| dt.with_timezone(&Local)))
            .map_err(|err| {
                rusqlite::Error::FromSqlConversionFailure(
                    11,
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
                is_unlimited_plan: row.get::<_, i64>(10)? != 0,
                queried_at,
                progress_percent: row.get(12)?,
            },
        ))
    })?;
    rows.collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(Into::into)
}

fn save_snapshots_tx(
    tx: &Transaction<'_>,
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
                is_unlimited_plan,
                queried_at,
                progress_percent
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
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
                snapshot.is_unlimited_plan,
                snapshot.queried_at.map(|value| value.to_rfc3339()),
                snapshot.progress_percent,
            ],
        )?;
    }
    Ok(())
}
