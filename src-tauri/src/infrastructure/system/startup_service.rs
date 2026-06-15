use crate::application::error::{AppError, AppResult};
use tauri_plugin_autostart::ManagerExt;

#[derive(Clone)]
pub struct StartupService {
    app_handle: tauri::AppHandle,
}

impl StartupService {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn set_launch_on_startup(&self, enabled: bool) -> AppResult<()> {
        let autolaunch = self.app_handle.autolaunch();
        if enabled {
            autolaunch
                .enable()
                .map_err(|err| AppError::System(format!("设置开机自启失败：{err}")))
        } else {
            autolaunch
                .disable()
                .map_err(|err| AppError::System(format!("关闭开机自启失败：{err}")))
        }
    }

    pub fn is_enabled(&self) -> AppResult<bool> {
        self.app_handle
            .autolaunch()
            .is_enabled()
            .map_err(|err| AppError::System(format!("读取开机自启状态失败：{err}")))
    }
}
