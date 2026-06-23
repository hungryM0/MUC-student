use std::env;
use std::path::PathBuf;

use muc_student_core::application::error::{AppError, AppResult};
use muc_student_core::application::platform::{RuntimePathProvider, StartupController};

pub struct TauriRuntimePathProvider;

impl RuntimePathProvider for TauriRuntimePathProvider {
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

#[cfg(windows)]
impl StartupController for RunKeyStartupController {
    fn set_launch_on_startup(&self, enabled: bool) -> AppResult<()> {
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
        use windows::Win32::System::Registry::{
            RegDeleteValueW, RegSetValueExW, KEY_SET_VALUE, REG_SZ,
        };

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
            close_key(key);
            result
        }
    }

    fn is_enabled(&self) -> AppResult<bool> {
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
        use windows::Win32::System::Registry::{RegQueryValueExW, KEY_READ};

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
            close_key(key);
            result
        }
    }
}

#[cfg(not(windows))]
impl StartupController for RunKeyStartupController {
    fn set_launch_on_startup(&self, _enabled: bool) -> AppResult<()> {
        Ok(())
    }

    fn is_enabled(&self) -> AppResult<bool> {
        Ok(false)
    }
}

#[cfg(windows)]
unsafe fn open_run_key(
    access: windows::Win32::System::Registry::REG_SAM_FLAGS,
) -> AppResult<windows::Win32::System::Registry::HKEY> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegCreateKeyExW, HKEY, HKEY_CURRENT_USER, REG_OPTION_NON_VOLATILE,
    };

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

#[cfg(windows)]
fn win32_ok(code: windows::Win32::Foundation::WIN32_ERROR, context: &str) -> AppResult<()> {
    use windows::Win32::Foundation::ERROR_SUCCESS;

    if code == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(AppError::System(format!("{context}：{code:?}")))
    }
}

#[cfg(windows)]
unsafe fn close_key(key: windows::Win32::System::Registry::HKEY) {
    use windows::Win32::System::Registry::RegCloseKey;

    let _ = RegCloseKey(key);
}

#[cfg(windows)]
fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
