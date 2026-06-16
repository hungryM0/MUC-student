pub mod application;
pub mod domain;
pub mod infrastructure;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

use crate::application::backend::{AccountInput, AccountUpdateInput, Backend, PreferenceInput};
use crate::application::error::{AppErrorDto, IntoCommandResult};

#[tauri::command(rename = "bootstrapApp", rename_all = "camelCase")]
async fn bootstrap_app(
    state: tauri::State<'_, Backend>,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.bootstrap_app().await.into_command_result()
}

#[tauri::command(rename = "selectAccount", rename_all = "camelCase")]
async fn select_account(
    state: tauri::State<'_, Backend>,
    account_id: String,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.select_account(account_id).await.into_command_result()
}

#[tauri::command(rename = "createAccount", rename_all = "camelCase")]
async fn create_account(
    state: tauri::State<'_, Backend>,
    input: AccountInput,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.create_account(input).await.into_command_result()
}

#[tauri::command(rename = "updateAccount", rename_all = "camelCase")]
async fn update_account(
    state: tauri::State<'_, Backend>,
    input: AccountUpdateInput,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.update_account(input).await.into_command_result()
}

#[tauri::command(rename = "deleteAccount", rename_all = "camelCase")]
async fn delete_account(
    state: tauri::State<'_, Backend>,
    account_id: String,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.delete_account(account_id).await.into_command_result()
}

#[tauri::command(rename = "loginSelectedAccount", rename_all = "camelCase")]
async fn login_selected_account(
    state: tauri::State<'_, Backend>,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.login_selected_account().await.into_command_result()
}

#[tauri::command(rename = "refreshDashboard", rename_all = "camelCase")]
async fn refresh_dashboard(
    state: tauri::State<'_, Backend>,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.refresh_dashboard().await.into_command_result()
}

#[tauri::command(rename = "logoutLocalDevice", rename_all = "camelCase")]
async fn logout_local_device(
    state: tauri::State<'_, Backend>,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.logout_local_device().await.into_command_result()
}

#[tauri::command(rename = "updatePreferences", rename_all = "camelCase")]
async fn update_preferences(
    state: tauri::State<'_, Backend>,
    input: PreferenceInput,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.update_preferences(input).await.into_command_result()
}

#[tauri::command(rename = "getAppSnapshot", rename_all = "camelCase")]
async fn get_app_snapshot(
    state: tauri::State<'_, Backend>,
) -> Result<crate::application::AppSnapshotDto, AppErrorDto> {
    state.get_snapshot().into_command_result()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let backend = Backend::build(app.handle().clone()).map_err(|err| err.to_string())?;
            app.manage(backend);

            let app_handle = app.handle().clone();
            let show_item = MenuItemBuilder::with_id("show", "显示主窗口").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出程序").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .items(&[&show_item, &quit_item])
                .build()?;
            TrayIconBuilder::new()
                .tooltip("MUC-student")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            if let Some(window) = app_handle.get_webview_window("main") {
                let backend = app_handle.state::<Backend>().inner().clone();
                let window_for_close = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let should_minimize_to_tray = {
                            let snapshot = backend.get_snapshot().ok();
                            snapshot
                                .map(|item| item.preferences.minimize_to_tray_on_close)
                                .unwrap_or(false)
                        };
                        if should_minimize_to_tray {
                            api.prevent_close();
                            let _ = window_for_close.hide();
                        }
                    }
                });
                let _ = window.show();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap_app,
            select_account,
            create_account,
            update_account,
            delete_account,
            login_selected_account,
            refresh_dashboard,
            logout_local_device,
            update_preferences,
            get_app_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
