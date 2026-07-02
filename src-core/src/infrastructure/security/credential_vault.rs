use std::collections::HashMap;
#[cfg(target_os = "android")]
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

use keyring::Entry;

use crate::application::error::{AppError, AppResult};

pub trait CredentialVault: Send + Sync {
    fn get_password(&self, account_id: &str) -> AppResult<String>;
    fn set_password(&self, account_id: &str, password: &str) -> AppResult<()>;
    fn delete_password(&self, account_id: &str) -> AppResult<()>;
}

#[derive(Clone, Default)]
pub struct SystemCredentialVault;

impl SystemCredentialVault {
    pub fn initialize() -> AppResult<Self> {
        Ok(Self)
    }

    #[cfg(target_os = "android")]
    fn ensure_default_store() -> AppResult<()> {
        static INIT: OnceLock<Result<(), String>> = OnceLock::new();

        INIT.get_or_init(|| {
            let mut config = HashMap::new();
            config.insert("name", "muc-student");
            android_native_keyring_store::Store::new_with_configuration(&config)
                .map(|store| keyring_core::set_default_store(store))
                .map_err(|err| format!("初始化 Android 凭据库失败：{err}"))
        })
        .clone()
        .map_err(AppError::System)
    }

    #[cfg(not(target_os = "android"))]
    fn ensure_default_store() -> AppResult<()> {
        Ok(())
    }

    fn entry(account_id: &str) -> AppResult<Entry> {
        Self::ensure_default_store()?;
        Entry::new("MUC-student", &format!("muc-student:{account_id}"))
            .map_err(|err| AppError::System(format!("打开凭据项失败：{err}")))
    }
}

impl CredentialVault for SystemCredentialVault {
    fn get_password(&self, account_id: &str) -> AppResult<String> {
        Self::entry(account_id)?
            .get_password()
            .map_err(|err| AppError::System(format!("读取系统凭据失败：{err}")))
    }

    fn set_password(&self, account_id: &str, password: &str) -> AppResult<()> {
        Self::entry(account_id)?
            .set_password(password)
            .map_err(|err| AppError::System(format!("写入系统凭据失败：{err}")))
    }

    fn delete_password(&self, account_id: &str) -> AppResult<()> {
        match Self::entry(account_id)?.delete_credential() {
            Ok(()) => Ok(()),
            Err(err) => {
                let text = err.to_string().to_lowercase();
                if text.contains("noentry")
                    || text.contains("not found")
                    || text.contains("notfound")
                {
                    Ok(())
                } else {
                    Err(AppError::System(format!("删除系统凭据失败：{err}")))
                }
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct MemoryCredentialVault {
    inner: Arc<Mutex<HashMap<String, String>>>,
}

impl CredentialVault for MemoryCredentialVault {
    fn get_password(&self, account_id: &str) -> AppResult<String> {
        self.inner
            .lock()
            .map_err(|_| AppError::System("凭据测试仓库锁损坏".to_string()))?
            .get(account_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("找不到账号密码".to_string()))
    }

    fn set_password(&self, account_id: &str, password: &str) -> AppResult<()> {
        self.inner
            .lock()
            .map_err(|_| AppError::System("凭据测试仓库锁损坏".to_string()))?
            .insert(account_id.to_string(), password.to_string());
        Ok(())
    }

    fn delete_password(&self, account_id: &str) -> AppResult<()> {
        self.inner
            .lock()
            .map_err(|_| AppError::System("凭据测试仓库锁损坏".to_string()))?
            .remove(account_id);
        Ok(())
    }
}
