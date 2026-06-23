mod platform;
mod plugins;

use std::sync::Arc;

use muc_student_core::application::{
    AppCore, AppErrorDto, AppSnapshotDto, IntoCommandResult, NoopEventSink,
};
use platform::{RunKeyStartupController, TauriRuntimePathProvider};
use tauri::Manager;

#[derive(Clone)]
struct ManagedAppCore {
    core: Arc<AppCore>,
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
fn update_tray_menu(
    app: tauri::AppHandle,
    show_text: String,
    quit_text: String,
) -> Result<(), String> {
    plugins::system_tray::update_tray_menu(&app, &show_text, &quit_text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .setup(|app| {
            let core = AppCore::build(
                Arc::new(TauriRuntimePathProvider),
                Arc::new(RunKeyStartupController::new("MUC-student")),
                Arc::new(NoopEventSink),
            )?;
            app.manage(ManagedAppCore {
                core: Arc::new(core),
            });
            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
                let _ = window.show();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(plugins::system_tray::init())
        .invoke_handler(tauri::generate_handler![
            bootstrap_app,
            get_app_snapshot,
            select_account,
            login_selected_account,
            refresh_dashboard,
            logout_local_device,
            update_tray_menu
        ]);

    #[cfg(not(debug_assertions))]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
