use std::path::Path;

use crate::application::error::AppResult;

pub fn write_json_atomic<T: serde::Serialize>(path: &Path, payload: &T) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state.json");
    let tmp_path = path.with_file_name(format!(".{file_name}.tmp"));
    let text = serde_json::to_string_pretty(payload)?;
    std::fs::write(&tmp_path, text)?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    std::fs::rename(tmp_path, path)?;
    Ok(())
}

pub fn read_json<T: serde::de::DeserializeOwned + Default>(path: &Path) -> AppResult<T> {
    if !path.exists() {
        return Ok(T::default());
    }
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}
