use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;

use crate::application::error::{AppError, AppResult};
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;

#[derive(Clone)]
pub struct AppDatabase {
    connection: Arc<Mutex<Connection>>,
}

impl AppDatabase {
    pub fn open(paths: &RuntimePaths) -> AppResult<Self> {
        let path = paths.database_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        connection.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA busy_timeout = 5000;

            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                remark_name TEXT NOT NULL,
                username TEXT NOT NULL UNIQUE,
                sort_order INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS app_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                last_login_time TEXT,
                last_quota_refresh_time TEXT,
                last_login_result TEXT NOT NULL,
                last_login_message TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS preferences (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                minimize_to_tray_on_close INTEGER NOT NULL,
                launch_on_startup INTEGER NOT NULL,
                auto_switch_account_on_traffic_exhausted INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS selection_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                selected_account_id TEXT NOT NULL,
                current_online_account_id TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS recent_accounts (
                account_id TEXT PRIMARY KEY,
                sort_order INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS status_card_order (
                account_id TEXT PRIMARY KEY,
                sort_order INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS traffic_snapshots (
                account_id TEXT PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
                used_traffic_text TEXT NOT NULL,
                product_balance_text TEXT NOT NULL,
                included_package_text TEXT NOT NULL,
                online_device_count_text TEXT NOT NULL,
                package_text TEXT NOT NULL,
                status_text TEXT NOT NULL,
                detail_text TEXT NOT NULL,
                queried_at TEXT,
                progress_percent REAL
            );

            CREATE TABLE IF NOT EXISTS panel_cookies (
                account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY (account_id, name)
            );

            INSERT OR IGNORE INTO app_state (
                id,
                last_login_result,
                last_login_message
            ) VALUES (1, '未执行', '-');

            INSERT OR IGNORE INTO preferences (
                id,
                minimize_to_tray_on_close,
                launch_on_startup,
                auto_switch_account_on_traffic_exhausted
            ) VALUES (1, 1, 0, 0);

            INSERT OR IGNORE INTO selection_state (
                id,
                selected_account_id,
                current_online_account_id
            ) VALUES (1, '', '');
            "#,
        )?;
        let version: u32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version < 1 {
            connection.execute_batch(
                r#"
                ALTER TABLE traffic_snapshots ADD COLUMN package_total_text TEXT NOT NULL DEFAULT '';
                ALTER TABLE traffic_snapshots ADD COLUMN package_available_text TEXT NOT NULL DEFAULT '';
                PRAGMA user_version = 1;
                "#,
            )?;
        }
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn lock(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| AppError::Storage("SQLite 连接锁损坏".to_string()))
    }
}
