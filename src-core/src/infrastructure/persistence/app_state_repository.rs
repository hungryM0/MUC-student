use chrono::{DateTime, Local};
use rusqlite::params;

use crate::application::error::AppResult;
use crate::domain::models::{AppState, UserPreferences};
use crate::infrastructure::persistence::database::AppDatabase;

#[derive(Clone)]
pub struct AppStateRepository {
    db: AppDatabase,
}

impl AppStateRepository {
    pub fn new(db: AppDatabase) -> Self {
        Self { db }
    }

    pub fn load_state(&self) -> AppResult<AppState> {
        let conn = self.db.lock()?;
        let (last_login_time, last_quota_refresh_time, last_login_result, last_login_message) =
            conn.query_row(
                r#"
                SELECT
                    last_login_time,
                    last_quota_refresh_time,
                    last_login_result,
                    last_login_message
                FROM app_state
                WHERE id = 1
                "#,
                [],
                |row| {
                    Ok((
                        parse_datetime(row.get::<_, Option<String>>(0)?)?,
                        parse_datetime(row.get::<_, Option<String>>(1)?)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )?;
        Ok(AppState {
            last_login_time,
            last_quota_refresh_time,
            last_login_result: if last_login_result.trim().is_empty() {
                "未执行".to_string()
            } else {
                last_login_result
            },
            last_login_message: if last_login_message.trim().is_empty() {
                "-".to_string()
            } else {
                last_login_message
            },
            recent_account_ids: normalize_recent_account_ids(load_recent_account_ids(&conn)?),
        })
    }

    pub fn load_preferences(&self) -> AppResult<UserPreferences> {
        let conn = self.db.lock()?;
        let preferences = conn.query_row(
            r#"
            SELECT
                minimize_to_tray_on_close,
                launch_on_startup,
                auto_switch_account_on_traffic_exhausted
            FROM preferences
            WHERE id = 1
            "#,
            [],
            |row| {
                Ok(UserPreferences {
                    minimize_to_tray_on_close: row.get::<_, bool>(0)?,
                    launch_on_startup: row.get::<_, bool>(1)?,
                    auto_switch_account_on_traffic_exhausted: row.get::<_, bool>(2)?,
                })
            },
        )?;
        Ok(preferences)
    }

    pub fn save_state(&self, state: &AppState) -> AppResult<()> {
        let mut conn = self.db.lock()?;
        let tx = conn.transaction()?;
        tx.execute(
            r#"
            UPDATE app_state
            SET
                last_login_time = ?1,
                last_quota_refresh_time = ?2,
                last_login_result = ?3,
                last_login_message = ?4
            WHERE id = 1
            "#,
            params![
                state.last_login_time.map(|value| value.to_rfc3339()),
                state
                    .last_quota_refresh_time
                    .map(|value| value.to_rfc3339()),
                state.last_login_result,
                state.last_login_message,
            ],
        )?;
        save_recent_account_ids_tx(&tx, &state.recent_account_ids)?;
        tx.commit()?;
        Ok(())
    }

    pub fn save_preferences(&self, preferences: &UserPreferences) -> AppResult<()> {
        let conn = self.db.lock()?;
        conn.execute(
            r#"
            UPDATE preferences
            SET
                minimize_to_tray_on_close = ?1,
                launch_on_startup = ?2,
                auto_switch_account_on_traffic_exhausted = ?3
            WHERE id = 1
            "#,
            params![
                preferences.minimize_to_tray_on_close,
                preferences.launch_on_startup,
                preferences.auto_switch_account_on_traffic_exhausted,
            ],
        )?;
        Ok(())
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
}

fn parse_datetime(input: Option<String>) -> rusqlite::Result<Option<DateTime<Local>>> {
    input
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map(|value| value.map(|dt| dt.with_timezone(&Local)))
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
        })
}

fn load_recent_account_ids(conn: &rusqlite::Connection) -> AppResult<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT account_id FROM recent_accounts ORDER BY sort_order, rowid")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn save_recent_account_ids_tx(
    tx: &rusqlite::Transaction<'_>,
    account_ids: &[String],
) -> AppResult<()> {
    tx.execute("DELETE FROM recent_accounts", [])?;
    for (index, account_id) in normalize_recent_account_ids(account_ids.to_vec())
        .iter()
        .enumerate()
    {
        tx.execute(
            "INSERT INTO recent_accounts (account_id, sort_order) VALUES (?1, ?2)",
            params![account_id, index as i64],
        )?;
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::AppStateRepository;
    use crate::infrastructure::persistence::database::AppDatabase;
    use crate::infrastructure::persistence::runtime_paths::RuntimePaths;

    #[test]
    fn stores_preferences_and_recent_accounts_in_sqlite() {
        let root = tempdir().expect("create temp dir");
        let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
        let db = AppDatabase::open(&paths).expect("open db");
        let repo = AppStateRepository::new(db);

        let mut preferences = repo.load_preferences().expect("load preferences");
        preferences.minimize_to_tray_on_close = true;
        preferences.auto_switch_account_on_traffic_exhausted = true;
        repo.save_preferences(&preferences)
            .expect("save preferences");

        repo.mark_account_used("acc-1").expect("mark first");
        repo.mark_account_used("acc-2").expect("mark second");
        repo.mark_account_used("acc-1").expect("mark first again");

        let state = repo.load_state().expect("load state");
        let preferences = repo.load_preferences().expect("reload preferences");
        assert_eq!(state.recent_account_ids, vec!["acc-1", "acc-2"]);
        assert!(preferences.minimize_to_tray_on_close);
        assert!(preferences.auto_switch_account_on_traffic_exhausted);
    }
}
