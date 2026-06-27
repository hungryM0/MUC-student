use std::path::{Path, PathBuf};

use crate::application::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct RuntimePaths {
    app_data_dir: PathBuf,
    resource_base_dir: PathBuf,
}

impl RuntimePaths {
    pub fn new(app_data_dir: PathBuf, resource_base_dir: PathBuf) -> AppResult<Self> {
        std::fs::create_dir_all(&app_data_dir)?;
        Ok(Self {
            app_data_dir,
            resource_base_dir,
        })
    }

    pub fn from_cwd_for_tests(root: impl Into<PathBuf>) -> AppResult<Self> {
        let root = root.into();
        Self::new(root.clone(), root)
    }

    pub fn database_path(&self) -> PathBuf {
        self.app_data_dir.join("muc_student.sqlite3")
    }

    pub fn app_data_dir(&self) -> &Path {
        &self.app_data_dir
    }

    pub fn resource_base_dir(&self) -> &Path {
        &self.resource_base_dir
    }
}

pub fn resolve_default_paths() -> AppResult<RuntimePaths> {
    let app_data = dirs::data_local_dir()
        .ok_or_else(|| AppError::Storage("无法定位 Windows 本地应用数据目录".to_string()))?
        .join("MUC-student");
    let resource_base = std::env::current_dir()?;
    RuntimePaths::new(app_data, resource_base)
}
