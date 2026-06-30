mod platform;
mod plugins;

use std::sync::Arc;

use muc_student_core::application::{
    AppCore, AppError, AppErrorDto, AppEventSink, AppResult, AppSnapshotDto, IntoCommandResult,
};
use platform::{RunKeyStartupController, TauriRuntimePathProvider};
use tauri::{Emitter, Manager};

#[derive(Clone)]
struct ManagedAppCore {
    core: Arc<AppCore>,
}

#[derive(Clone)]
struct TauriEventSink {
    app: tauri::AppHandle,
}

impl TauriEventSink {
    fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

impl AppEventSink for TauriEventSink {
    fn state_updated(&self, snapshot: &AppSnapshotDto) -> AppResult<()> {
        self.app
            .emit("muc://state-updated", snapshot.clone())
            .map_err(|err| AppError::System(err.to_string()))
    }

    fn task_started(&self, task: &str) -> AppResult<()> {
        self.app
            .emit("muc://task-started", task.to_string())
            .map_err(|err| AppError::System(err.to_string()))
    }

    fn task_finished(&self, task: &str) -> AppResult<()> {
        self.app
            .emit("muc://task-finished", task.to_string())
            .map_err(|err| AppError::System(err.to_string()))
    }
}

#[tauri::command]
async fn bootstrap_app(
    core: tauri::State<'_, ManagedAppCore>,
) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core.bootstrap_app().await.into_command_result()
}

#[tauri::command]
fn get_app_snapshot(core: tauri::State<'_, ManagedAppCore>) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core.get_snapshot().into_command_result()
}

#[tauri::command]
async fn select_account(
    core: tauri::State<'_, ManagedAppCore>,
    account_id: String,
) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core
        .select_account(account_id)
        .await
        .into_command_result()
}

#[tauri::command]
async fn add_account(
    core: tauri::State<'_, ManagedAppCore>,
    remark_name: String,
    username: String,
    password: String,
) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core
        .add_account(remark_name, username, password)
        .await
        .into_command_result()
}

#[tauri::command]
async fn update_account(
    core: tauri::State<'_, ManagedAppCore>,
    account_id: String,
    remark_name: String,
    username: String,
    password: Option<String>,
) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core
        .update_account(account_id, remark_name, username, password)
        .await
        .into_command_result()
}

#[tauri::command]
async fn delete_account(
    core: tauri::State<'_, ManagedAppCore>,
    account_id: String,
) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core
        .delete_account(account_id)
        .await
        .into_command_result()
}

#[tauri::command]
async fn login_selected_account(
    core: tauri::State<'_, ManagedAppCore>,
) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core
        .login_selected_account()
        .await
        .into_command_result()
}

#[tauri::command]
async fn refresh_dashboard(
    core: tauri::State<'_, ManagedAppCore>,
) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core.refresh_dashboard().await.into_command_result()
}

#[tauri::command]
async fn logout_local_device(
    core: tauri::State<'_, ManagedAppCore>,
) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core.logout_local_device().await.into_command_result()
}

#[tauri::command]
async fn update_preferences(
    core: tauri::State<'_, ManagedAppCore>,
    minimize_to_tray_on_close: bool,
    launch_on_startup: bool,
    auto_switch_account_on_traffic_exhausted: bool,
) -> Result<AppSnapshotDto, AppErrorDto> {
    core.core
        .update_preferences(
            minimize_to_tray_on_close,
            launch_on_startup,
            auto_switch_account_on_traffic_exhausted,
        )
        .await
        .into_command_result()
}

#[tauri::command]
#[cfg(desktop)]
fn update_tray_menu(
    app: tauri::AppHandle,
    show_text: String,
    quit_text: String,
) -> Result<(), String> {
    plugins::system_tray::update_tray_menu(&app, &show_text, &quit_text)
}

#[tauri::command]
#[cfg(mobile)]
fn update_tray_menu(_show_text: String, _quit_text: String) -> Result<(), String> {
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().setup(|app| {
        let core = AppCore::build(
            Arc::new(TauriRuntimePathProvider::new(app.handle().clone())),
            Arc::new(RunKeyStartupController::new("MUC-student")),
            Arc::new(TauriEventSink::new(app.handle().clone())),
        )?;
        app.manage(ManagedAppCore {
            core: Arc::new(core),
        });
        Ok(())
    });

    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_focus();
            let _ = window.unminimize();
            let _ = window.show();
        }
    }));

    let builder = builder.plugin(tauri_plugin_opener::init());

    #[cfg(desktop)]
    let builder = builder.plugin(plugins::system_tray::init());

    #[cfg(desktop)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        bootstrap_app,
        get_app_snapshot,
        select_account,
        add_account,
        update_account,
        delete_account,
        login_selected_account,
        refresh_dashboard,
        logout_local_device,
        update_preferences,
        update_tray_menu
    ]);

    #[cfg(mobile)]
    let builder = builder.invoke_handler(tauri::generate_handler![
        bootstrap_app,
        get_app_snapshot,
        select_account,
        add_account,
        update_account,
        delete_account,
        login_selected_account,
        refresh_dashboard,
        logout_local_device,
        update_preferences,
        update_tray_menu
    ]);

    #[cfg(all(desktop, not(debug_assertions)))]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
