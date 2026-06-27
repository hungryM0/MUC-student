use std::path::PathBuf;

use crate::application::dto::AppSnapshotDto;
use crate::application::error::AppResult;

pub trait RuntimePathProvider: Send + Sync {
    fn app_data_dir(&self) -> AppResult<PathBuf>;
    fn resource_base_dir(&self) -> AppResult<PathBuf>;
}

pub trait StartupController: Send + Sync {
    fn set_launch_on_startup(&self, enabled: bool) -> AppResult<()>;
    fn is_enabled(&self) -> AppResult<bool>;
}

pub trait AppEventSink: Send + Sync {
    fn state_updated(&self, snapshot: &AppSnapshotDto) -> AppResult<()>;
    fn task_started(&self, task: &str) -> AppResult<()>;
    fn task_finished(&self, task: &str) -> AppResult<()>;
}

#[derive(Clone, Default)]
pub struct NoopEventSink;

impl AppEventSink for NoopEventSink {
    fn state_updated(&self, _snapshot: &AppSnapshotDto) -> AppResult<()> {
        Ok(())
    }

    fn task_started(&self, _task: &str) -> AppResult<()> {
        Ok(())
    }

    fn task_finished(&self, _task: &str) -> AppResult<()> {
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct NoopStartupController;

impl StartupController for NoopStartupController {
    fn set_launch_on_startup(&self, _enabled: bool) -> AppResult<()> {
        Ok(())
    }

    fn is_enabled(&self) -> AppResult<bool> {
        Ok(false)
    }
}
