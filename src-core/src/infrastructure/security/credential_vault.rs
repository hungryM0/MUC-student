use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use keyring::Entry;

use crate::application::error::{AppError, AppResult};

pub trait CredentialVault: Send + Sync {
    fn get_password(&self, account_id: &str) -> AppResult<String>;
    fn set_password(&self, account_id: &str, password: &str) -> AppResult<()>;
    fn delete_password(&self, account_id: &str) -> AppResult<()>;
}

#[derive(Clone, Default)]
pub struct WindowsCredentialVault;

impl WindowsCredentialVault {
    pub fn initialize() -> AppResult<Self> {
        Ok(Self)
    }

    fn entry(account_id: &str) -> AppResult<Entry> {
        Entry::new("MUC-student", &format!("muc-student:{account_id}"))
            .map_err(|err| AppError::System(format!("打开凭据项失败：{err}")))
    }
}

impl CredentialVault for WindowsCredentialVault {
    fn get_password(&self, account_id: &str) -> AppResult<String> {
        Self::entry(account_id)?
            .get_password()
            .map_err(|err| AppError::System(format!("读取 Windows 凭据失败：{err}")))
    }

    fn set_password(&self, account_id: &str, password: &str) -> AppResult<()> {
        Self::entry(account_id)?
            .set_password(password)
            .map_err(|err| AppError::System(format!("写入 Windows 凭据失败：{err}")))
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
                    Err(AppError::System(format!("删除 Windows 凭据失败：{err}")))
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
