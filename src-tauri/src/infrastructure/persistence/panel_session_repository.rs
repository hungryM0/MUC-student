use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::application::error::AppResult;
use crate::infrastructure::persistence::file_write::{read_json, write_json_atomic};
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;

#[derive(Clone)]
pub struct PanelSessionRepository {
    paths: RuntimePaths,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PanelSessionFile {
    sessions: BTreeMap<String, BTreeMap<String, String>>,
}

impl PanelSessionRepository {
    pub fn new(paths: RuntimePaths) -> Self {
        Self { paths }
    }

    pub fn load_session(&self, account_id: &str) -> AppResult<BTreeMap<String, String>> {
        let file = self.load_file()?;
        Ok(file
            .sessions
            .get(account_id.trim())
            .cloned()
            .unwrap_or_default())
    }

    pub fn save_session(
        &self,
        account_id: &str,
        cookies: &std::collections::HashMap<String, String>,
    ) -> AppResult<()> {
        let clean_id = account_id.trim();
        if clean_id.is_empty() {
            return Ok(());
        }
        let filtered = normalize_cookies(cookies);
        let mut file = self.load_file()?;
        if filtered.is_empty() {
            file.sessions.remove(clean_id);
        } else {
            file.sessions.insert(clean_id.to_string(), filtered);
        }
        self.save_file(&file)
    }

    pub fn clear_session(&self, account_id: &str) -> AppResult<()> {
        let clean_id = account_id.trim();
        if clean_id.is_empty() {
            return Ok(());
        }
        let mut file = self.load_file()?;
        file.sessions.remove(clean_id);
        self.save_file(&file)
    }

    fn load_file(&self) -> AppResult<PanelSessionFile> {
        read_json(&self.paths.panel_sessions_path())
    }

    fn save_file(&self, file: &PanelSessionFile) -> AppResult<()> {
        write_json_atomic(&self.paths.panel_sessions_path(), file)
    }
}

fn normalize_cookies(
    cookies: &std::collections::HashMap<String, String>,
) -> BTreeMap<String, String> {
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
