use std::collections::{BTreeMap, HashMap};

use rusqlite::params;

use crate::application::error::AppResult;
use crate::infrastructure::persistence::database::AppDatabase;

#[derive(Clone)]
pub struct PanelSessionRepository {
    db: AppDatabase,
}

impl PanelSessionRepository {
    pub fn new(db: AppDatabase) -> Self {
        Self { db }
    }

    pub fn load_session(&self, account_id: &str) -> AppResult<BTreeMap<String, String>> {
        let clean_id = account_id.trim();
        if clean_id.is_empty() {
            return Ok(BTreeMap::new());
        }
        let conn = self.db.lock()?;
        let mut stmt = conn
            .prepare("SELECT name, value FROM panel_cookies WHERE account_id = ?1 ORDER BY name")?;
        let rows = stmt.query_map(params![clean_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect::<Result<BTreeMap<_, _>, _>>()
            .map_err(Into::into)
    }

    pub fn save_session(
        &self,
        account_id: &str,
        cookies: &HashMap<String, String>,
    ) -> AppResult<()> {
        let clean_id = account_id.trim();
        if clean_id.is_empty() {
            return Ok(());
        }
        let filtered = normalize_cookies(cookies);
        let mut conn = self.db.lock()?;
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM panel_cookies WHERE account_id = ?1",
            params![clean_id],
        )?;
        for (name, value) in filtered {
            tx.execute(
                "INSERT INTO panel_cookies (account_id, name, value) VALUES (?1, ?2, ?3)",
                params![clean_id, name, value],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn clear_session(&self, account_id: &str) -> AppResult<()> {
        let clean_id = account_id.trim();
        if clean_id.is_empty() {
            return Ok(());
        }
        let conn = self.db.lock()?;
        conn.execute(
            "DELETE FROM panel_cookies WHERE account_id = ?1",
            params![clean_id],
        )?;
        Ok(())
    }
}

fn normalize_cookies(cookies: &HashMap<String, String>) -> BTreeMap<String, String> {
    cookies
        .iter()
        .filter_map(|(name, value)| {
            let clean_name = name.trim();
            let clean_value = value.trim();
            if clean_name.is_empty() || clean_value.is_empty() {
                None
            } else {
                Some((clean_name.to_string(), clean_value.to_string()))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tempfile::tempdir;

    use super::PanelSessionRepository;
    use crate::infrastructure::persistence::database::AppDatabase;
    use crate::infrastructure::persistence::runtime_paths::RuntimePaths;

    #[test]
    fn stores_and_clears_panel_cookies() {
        let root = tempdir().expect("create temp dir");
        let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
        let db = AppDatabase::open(&paths).expect("open db");
        let repo = PanelSessionRepository::new(db.clone());
        {
            let conn = db.lock().expect("lock db");
            conn.execute(
                "INSERT INTO accounts (id, remark_name, username, sort_order) VALUES ('acc-1', '主号', '20260001', 0)",
                [],
            )
            .expect("insert account");
        }

        let mut cookies = HashMap::new();
        cookies.insert("PHPSESSID_8800".to_string(), "abc".to_string());
        cookies.insert(" empty ".to_string(), " ".to_string());
        repo.save_session("acc-1", &cookies).expect("save session");

        let session = repo.load_session("acc-1").expect("load session");
        assert_eq!(
            session.get("PHPSESSID_8800").map(String::as_str),
            Some("abc")
        );
        assert!(!session.contains_key("empty"));

        repo.clear_session("acc-1").expect("clear session");
        assert!(repo
            .load_session("acc-1")
            .expect("reload session")
            .is_empty());
    }
}
