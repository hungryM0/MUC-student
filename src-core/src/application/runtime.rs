use std::sync::{Arc, RwLock};

use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::{AccountStore, AppState, NetworkStatus, UserPreferences};

#[derive(Clone, Debug, Default)]
pub struct AppRuntimeState {
    pub account_store: AccountStore,
    pub app_state: AppState,
    pub preferences: UserPreferences,
    pub network: NetworkStatus,
    pub snapshots: std::collections::BTreeMap<String, AccountTrafficSnapshot>,
    pub current_online_account_id: String,
    pub login_running: bool,
    pub refresh_running: bool,
    pub logout_running: bool,
}

#[derive(Clone, Default)]
pub struct SharedRuntimeState {
    inner: Arc<RwLock<AppRuntimeState>>,
}

impl SharedRuntimeState {
    pub fn new(state: AppRuntimeState) -> Self {
        Self {
            inner: Arc::new(RwLock::new(state)),
        }
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, AppRuntimeState> {
        self.inner.read().expect("runtime state lock poisoned")
    }

    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, AppRuntimeState> {
        self.inner.write().expect("runtime state lock poisoned")
    }
}
