use std::path::{Path, PathBuf};

use crate::application::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct RuntimePaths {
    app_data_dir: PathBuf,
    resource_base_dir: PathBuf,
    legacy_root: PathBuf,
}

impl RuntimePaths {
    pub fn new(
        app_data_dir: PathBuf,
        resource_base_dir: PathBuf,
        legacy_root: PathBuf,
    ) -> AppResult<Self> {
        std::fs::create_dir_all(&app_data_dir)?;
        Ok(Self {
            app_data_dir,
            resource_base_dir,
            legacy_root,
        })
    }

    pub fn from_cwd_for_tests(root: impl Into<PathBuf>) -> AppResult<Self> {
        let root = root.into();
        Self::new(root.clone(), root.clone(), root)
    }

    pub fn accounts_path(&self) -> PathBuf {
        self.app_data_dir.join("accounts.json")
    }

    pub fn app_state_path(&self) -> PathBuf {
        self.app_data_dir.join("app_state.json")
    }

    pub fn panel_sessions_path(&self) -> PathBuf {
        self.app_data_dir.join("panel_sessions.json")
    }

    pub fn legacy_accounts_path(&self) -> PathBuf {
        self.legacy_root.join("accounts.json")
    }

    pub fn legacy_app_state_path(&self) -> PathBuf {
        self.legacy_root.join("app_state.json")
    }

    pub fn app_data_dir(&self) -> &Path {
        &self.app_data_dir
    }

    pub fn legacy_root(&self) -> &Path {
        &self.legacy_root
    }

    pub fn resource_base_dir(&self) -> &Path {
        &self.resource_base_dir
    }

    pub fn ddddocr_model_path(&self) -> PathBuf {
        self.resource_base_dir
            .join("resources")
            .join("ocr")
            .join("common_old.onnx")
    }

    pub fn ocr_worker_path(&self) -> PathBuf {
        self.resource_base_dir
            .join("resources")
            .join("ocr")
            .join("ocr-worker.exe")
    }
}

pub fn resolve_default_paths() -> AppResult<RuntimePaths> {
    let app_data = dirs::data_local_dir()
        .ok_or_else(|| AppError::Storage("无法定位 Windows 本地应用数据目录".to_string()))?
        .join("MUC-student");
    let legacy_root = std::env::current_dir()?;
    RuntimePaths::new(app_data, legacy_root.clone(), legacy_root)
}
