#![allow(dead_code)]

use std::env;
use std::path::PathBuf;

use muc_student_core::application::error::{AppError, AppResult};
use muc_student_core::application::platform::{RuntimePathProvider, StartupController};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, HWND, WIN32_ERROR};
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SZ,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    LoadIconW, ShowWindow, IDI_APPLICATION, SW_HIDE, SW_RESTORE, WM_USER,
};
use windows_core::PCWSTR;

pub struct Win32RuntimePathProvider;

impl RuntimePathProvider for Win32RuntimePathProvider {
    fn app_data_dir(&self) -> AppResult<PathBuf> {
        dirs::data_local_dir()
            .map(|path| path.join("MUC-student"))
            .ok_or_else(|| AppError::Storage("无法定位 Windows 本地应用数据目录".to_string()))
    }

    fn resource_base_dir(&self) -> AppResult<PathBuf> {
        env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(PathBuf::from))
            .ok_or_else(|| AppError::Storage("无法定位程序资源目录".to_string()))
    }

    fn legacy_root(&self) -> AppResult<PathBuf> {
        env::current_dir().map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct RunKeyStartupController {
    app_name: String,
}

impl RunKeyStartupController {
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }
}

impl StartupController for RunKeyStartupController {
    fn set_launch_on_startup(&self, enabled: bool) -> AppResult<()> {
        let exe = env::current_exe()
            .map_err(|err| AppError::System(format!("读取程序路径失败：{err}")))?;
        let command = format!("\"{}\"", exe.display());
        let name = wide_null(&self.app_name);

        unsafe {
            let key = open_run_key(KEY_SET_VALUE)?;
            let result = if enabled {
                let mut value = wide_null(&command);
                let bytes = std::slice::from_raw_parts(
                    value.as_mut_ptr().cast::<u8>(),
                    value.len() * std::mem::size_of::<u16>(),
                );
                win32_ok(
                    RegSetValueExW(key, PCWSTR(name.as_ptr()), None, REG_SZ, Some(bytes)),
                    "设置开机自启失败",
                )
            } else {
                let code = RegDeleteValueW(key, PCWSTR(name.as_ptr()));
                if code == ERROR_FILE_NOT_FOUND {
                    Ok(())
                } else {
                    win32_ok(code, "关闭开机自启失败")
                }
            };
            let _ = RegCloseKey(key);
            result
        }
    }

    fn is_enabled(&self) -> AppResult<bool> {
        let name = wide_null(&self.app_name);
        unsafe {
            let key = open_run_key(KEY_READ)?;
            let code = RegQueryValueExW(key, PCWSTR(name.as_ptr()), None, None, None, None);
            let result = if code == ERROR_SUCCESS {
                Ok(true)
            } else if code == ERROR_FILE_NOT_FOUND {
                Ok(false)
            } else {
                Err(AppError::System(format!("读取开机自启状态失败：{code:?}")))
            };
            let _ = RegCloseKey(key);
            result
        }
    }
}

pub struct TrayIcon {
    hwnd: HWND,
    id: u32,
}

impl TrayIcon {
    pub fn add(hwnd: HWND) -> AppResult<Self> {
        let id = 1;
        let mut data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: id,
            uFlags: NIF_ICON | NIF_TIP | NIF_MESSAGE,
            uCallbackMessage: WM_USER + 1,
            ..Default::default()
        };
        data.hIcon = unsafe { LoadIconW(None, IDI_APPLICATION) }
            .map_err(|err| AppError::System(format!("加载托盘图标失败：{err}")))?;
        write_tip(&mut data, "MUC-student");

        if unsafe { Shell_NotifyIconW(NIM_ADD, &data) }.as_bool() {
            Ok(Self { hwnd, id })
        } else {
            Err(AppError::System("创建托盘图标失败".to_string()))
        }
    }

    #[allow(dead_code)]
    pub fn hide_window(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    #[allow(dead_code)]
    pub fn show_window(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_RESTORE);
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let data = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: self.id,
            ..Default::default()
        };
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &data);
        }
    }
}

unsafe fn open_run_key(access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> AppResult<HKEY> {
    let subkey = wide_null("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let mut key = HKEY::default();
    let code = RegCreateKeyExW(
        HKEY_CURRENT_USER,
        PCWSTR(subkey.as_ptr()),
        Some(0),
        None,
        REG_OPTION_NON_VOLATILE,
        access,
        None,
        &mut key,
        None,
    );
    win32_ok(code, "打开开机自启注册表失败")?;
    Ok(key)
}

fn win32_ok(code: WIN32_ERROR, context: &str) -> AppResult<()> {
    if code == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(AppError::System(format!("{context}：{code:?}")))
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn write_tip(data: &mut NOTIFYICONDATAW, value: &str) {
    let wide = wide_null(value);
    for (slot, ch) in data.szTip.iter_mut().zip(wide.into_iter()) {
        *slot = ch;
    }
}
