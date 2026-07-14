use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Local;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::application::account_pool_transfer::{
    decode_account_pool, encode_account_pool, AccountPoolEntry, AccountPoolState,
};
use crate::application::dto::{AccountPoolImportResultDto, AppSnapshotDto};
use crate::application::error::{AppError, AppResult};
use crate::application::platform::{AppEventSink, RuntimePathProvider, StartupController};
use crate::application::runtime::{AppRuntimeState, SharedRuntimeState};
use crate::application::runtime_refresh::refresh_runtime_from_disk;
use crate::application::services::dashboard_refresh_service::DashboardRefreshService;
use crate::application::services::session_service::SessionService;
use crate::application::services::snapshot_mapper::{
    build_app_snapshot, remove_expired_unlimited_snapshots, restore_cached_snapshots,
};
use crate::domain::models::{CachedTrafficSnapshot, NetworkStatus};
use crate::domain::policies::traffic_math::build_auto_switch_candidate;
use crate::infrastructure::android_keepalive::write_android_keepalive_state;
use crate::infrastructure::network::{
    http_transport::HttpTransport,
    legacy_portal_auth_client::LegacyPortalAuthClient,
    legacy_portal_status_client::LegacyPortalStatusClient,
    network_status_service::{NetworkStatusDetector, NetworkStatusService},
    self_service_panel_client::SelfServicePanelClient,
};
use crate::infrastructure::persistence::account_repository::{
    AccountImportRecord, AccountRepository,
};
use crate::infrastructure::persistence::account_snapshot_repository::AccountSnapshotRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;
use crate::infrastructure::persistence::database::AppDatabase;
use crate::infrastructure::persistence::panel_session_repository::PanelSessionRepository;
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
use crate::infrastructure::security::credential_vault::{CredentialVault, SystemCredentialVault};
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct AppCore {
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    app_state_repo: AppStateRepository,
    session_service: SessionService,
    dashboard_refresh_service: DashboardRefreshService,
    network_task_lock: Arc<Mutex<()>>,
    background_refresh_started: Arc<AtomicBool>,
    app_data_dir: std::path::PathBuf,
    event_sink: Arc<dyn AppEventSink>,
    startup_controller: Arc<dyn StartupController>,
}

impl AppCore {
    const QUOTA_REFRESH_COOLDOWN_MINUTES: i64 = 30;
    const BACKGROUND_REFRESH_INTERVAL_SECONDS: u64 = 5 * 60;

    pub fn build(
        path_provider: Arc<dyn RuntimePathProvider>,
        startup_controller: Arc<dyn StartupController>,
        event_sink: Arc<dyn AppEventSink>,
    ) -> AppResult<Self> {
        let settings = AppSettings::default();
        let network_status_service = Arc::new(NetworkStatusService::new(settings));
        Self::build_with_network_status_detector(
            path_provider,
            startup_controller,
            event_sink,
            network_status_service,
        )
    }

    pub fn build_with_network_status_detector(
        path_provider: Arc<dyn RuntimePathProvider>,
        startup_controller: Arc<dyn StartupController>,
        event_sink: Arc<dyn AppEventSink>,
        network_status_service: Arc<dyn NetworkStatusDetector>,
    ) -> AppResult<Self> {
        let paths = RuntimePaths::new(
            path_provider.app_data_dir()?,
            path_provider.resource_base_dir()?,
        )?;
        Self::build_with_paths_and_network_status_detector(
            paths,
            startup_controller,
            event_sink,
            network_status_service,
        )
    }

    pub fn build_with_paths(
        paths: RuntimePaths,
        startup_controller: Arc<dyn StartupController>,
        event_sink: Arc<dyn AppEventSink>,
    ) -> AppResult<Self> {
        let settings = AppSettings::default();
        let network_status_service = Arc::new(NetworkStatusService::new(settings));
        Self::build_with_paths_and_network_status_detector(
            paths,
            startup_controller,
            event_sink,
            network_status_service,
        )
    }

    fn build_with_paths_and_network_status_detector(
        paths: RuntimePaths,
        startup_controller: Arc<dyn StartupController>,
        event_sink: Arc<dyn AppEventSink>,
        network_status_service: Arc<dyn NetworkStatusDetector>,
    ) -> AppResult<Self> {
        let settings = AppSettings::default();
        let vault: Arc<dyn CredentialVault> = Arc::new(SystemCredentialVault::initialize()?);
        let db = AppDatabase::open(&paths)?;
        let account_repo = AccountRepository::new(db.clone(), vault.clone());
        let snapshot_repo = AccountSnapshotRepository::new(db.clone());
        let app_state_repo = AppStateRepository::new(db.clone());
        let mut account_store = account_repo.ensure_store()?;
        let had_current_online_account = !account_store.current_online_account_id.is_empty();
        if had_current_online_account {
            account_store.current_online_account_id.clear();
        }
        let removed_expired_snapshots =
            remove_expired_unlimited_snapshots(&mut account_store.cached_traffic_snapshots);
        if had_current_online_account || removed_expired_snapshots {
            snapshot_repo.save_store_state(&account_store)?;
        }
        let app_state = app_state_repo.load_state()?;
        let mut preferences = app_state_repo.load_preferences()?;
        preferences.launch_on_startup = startup_controller.is_enabled()?;
        let panel_session_repo = PanelSessionRepository::new(db.clone());

        let transport = HttpTransport::new(settings.clone())?;
        let auth_client = LegacyPortalAuthClient::new(settings.clone(), transport.clone());
        let portal_status_client =
            LegacyPortalStatusClient::new(settings.clone(), transport.clone());
        let panel_client =
            SelfServicePanelClient::new(settings.clone(), transport, panel_session_repo);
        let snapshots = restore_cached_snapshots(&account_store.cached_traffic_snapshots);
        let runtime = SharedRuntimeState::new(AppRuntimeState {
            account_store: account_store.clone(),
            app_state: app_state.clone(),
            preferences: preferences.clone(),
            network: NetworkStatus::default(),
            snapshots,
            current_online_account_id: account_store.current_online_account_id.clone(),
            login_running: false,
            refresh_running: false,
            logout_running: false,
        });
        let session_service = SessionService::new(
            runtime.clone(),
            account_repo.clone(),
            snapshot_repo.clone(),
            app_state_repo.clone(),
            auth_client.clone(),
            portal_status_client.clone(),
            network_status_service.clone(),
        );
        let dashboard_refresh_service = DashboardRefreshService::new(
            runtime.clone(),
            account_repo.clone(),
            snapshot_repo,
            app_state_repo.clone(),
            portal_status_client,
            panel_client.clone(),
            network_status_service,
            event_sink.clone(),
        );
        let backend = Self {
            state: runtime,
            account_repo,
            app_state_repo,
            session_service,
            dashboard_refresh_service,
            network_task_lock: Arc::new(Mutex::new(())),
            background_refresh_started: Arc::new(AtomicBool::new(false)),
            app_data_dir: paths.app_data_dir().to_path_buf(),
            event_sink,
            startup_controller,
        };
        Ok(backend)
    }

    pub async fn bootstrap_app(&self) -> AppResult<AppSnapshotDto> {
        self.refresh_runtime_from_disk()?;
        self.start_background_refresh_loop();
        let snapshot = self.emit_state()?;
        let backend = self.clone();
        tokio::spawn(async move {
            let _ = backend.refresh_dashboard_silently().await;
        });
        Ok(snapshot)
    }

    pub fn get_snapshot(&self) -> AppResult<AppSnapshotDto> {
        self.build_snapshot()
    }

    pub async fn select_account(&self, account_id: String) -> AppResult<AppSnapshotDto> {
        let account = self.account_repo.select_account(&account_id)?;
        self.app_state_repo.mark_account_used(&account.id)?;
        self.refresh_runtime_from_disk()?;
        self.emit_state()
    }

    pub async fn add_account(
        &self,
        remark_name: String,
        username: String,
        password: String,
    ) -> AppResult<AppSnapshotDto> {
        self.validate_account_input(&remark_name, &username, &password)?;
        let account = self
            .account_repo
            .add_account(&remark_name, &username, &password)?;
        self.app_state_repo.mark_account_used(&account.id)?;
        self.refresh_runtime_from_disk()?;
        self.emit_state()
    }

    pub async fn update_account(
        &self,
        account_id: String,
        remark_name: String,
        username: String,
        password: Option<String>,
    ) -> AppResult<AppSnapshotDto> {
        self.validate_account_update_input(&remark_name, &username, password.as_deref())?;
        let account = self.account_repo.update_account(
            &account_id,
            &remark_name,
            &username,
            password.as_deref(),
        )?;
        self.app_state_repo.mark_account_used(&account.id)?;
        self.refresh_runtime_from_disk()?;
        self.emit_state()
    }
    pub async fn delete_account(&self, account_id: String) -> AppResult<AppSnapshotDto> {
        self.account_repo.delete_account(&account_id)?;
        let store = self.account_repo.load_store()?;
        let valid_ids = store
            .accounts
            .iter()
            .map(|account| account.id.clone())
            .collect::<std::collections::HashSet<_>>();
        self.app_state_repo.prune_recent_account_ids(&valid_ids)?;
        self.refresh_runtime_from_disk()?;
        self.emit_state()
    }

    pub async fn export_account_pool(&self, passphrase: String) -> AppResult<String> {
        let store = self.account_repo.load_store()?;
        let accounts = store
            .accounts
            .iter()
            .map(|account| {
                self.account_repo
                    .load_account_with_password(account)
                    .map(|item| AccountPoolEntry {
                        remark_name: item.account.remark_name,
                        username: item.account.username,
                        password: item.password,
                        cached_traffic_snapshot: store
                            .cached_traffic_snapshots
                            .get(&item.account.id)
                            .cloned(),
                    })
            })
            .collect::<AppResult<Vec<_>>>()?;
        let current_online_username = store
            .accounts
            .iter()
            .find(|account| account.id == store.current_online_account_id)
            .map(|account| account.username.clone());
        let status_card_order_usernames = store
            .status_card_order_snapshot
            .iter()
            .filter_map(|account_id| {
                store
                    .accounts
                    .iter()
                    .find(|account| account.id == *account_id)
                    .map(|account| account.username.clone())
            })
            .collect();
        let state = AccountPoolState {
            current_online_username,
            status_card_order_usernames,
        };

        if accounts.is_empty() {
            return Err(AppError::Validation("没有可导出的账号".to_string()));
        }
        encode_account_pool(accounts, state, &passphrase)
    }

    pub async fn import_account_pool(
        &self,
        code: String,
        passphrase: String,
    ) -> AppResult<AccountPoolImportResultDto> {
        let pool = decode_account_pool(&code, &passphrase)?;
        let records = pool
            .accounts
            .iter()
            .map(|item| AccountImportRecord {
                remark_name: item.remark_name.clone(),
                username: item.username.clone(),
                password: item.password.clone(),
            })
            .collect();
        let import_usernames = pool
            .accounts
            .iter()
            .map(|item| item.username.clone())
            .collect::<Vec<_>>();
        let snapshots_by_username = pool
            .accounts
            .iter()
            .filter_map(|item| {
                item.cached_traffic_snapshot
                    .clone()
                    .map(|snapshot| (item.username.clone(), snapshot))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let stats = self.account_repo.import_accounts(records)?;
        self.apply_imported_account_pool_state(
            &import_usernames,
            &snapshots_by_username,
            pool.state,
        )?;
        self.refresh_runtime_from_disk()?;
        let snapshot = self.emit_state()?;
        Ok(AccountPoolImportResultDto {
            snapshot,
            imported_count: stats.imported_count,
            overwritten_count: stats.overwritten_count,
        })
    }

    fn apply_imported_account_pool_state(
        &self,
        import_usernames: &[String],
        snapshots_by_username: &std::collections::BTreeMap<String, CachedTrafficSnapshot>,
        pool_state: AccountPoolState,
    ) -> AppResult<()> {
        let mut store = self.account_repo.load_store()?;
        let id_by_username = store
            .accounts
            .iter()
            .map(|account| (account.username.clone(), account.id.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();

        let imported_username_set = import_usernames
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        let mut sorted_accounts = Vec::with_capacity(store.accounts.len());
        for username in import_usernames {
            if let Some(index) = store
                .accounts
                .iter()
                .position(|account| account.username == *username)
            {
                sorted_accounts.push(store.accounts.remove(index));
            }
        }
        sorted_accounts.extend(store.accounts);
        store.accounts = sorted_accounts;

        for (username, snapshot) in snapshots_by_username {
            if let Some(account_id) = id_by_username.get(username) {
                store
                    .cached_traffic_snapshots
                    .insert(account_id.clone(), snapshot.clone());
            }
        }
        for account in &store.accounts {
            if imported_username_set.contains(&account.username)
                && !snapshots_by_username.contains_key(&account.username)
            {
                store.cached_traffic_snapshots.remove(&account.id);
            }
        }

        store.status_card_order_snapshot = pool_state
            .status_card_order_usernames
            .into_iter()
            .filter_map(|username| id_by_username.get(&username).cloned())
            .collect();
        store.current_online_account_id = pool_state
            .current_online_username
            .and_then(|username| id_by_username.get(&username).cloned())
            .unwrap_or_default();

        self.account_repo.save_store(&store)
    }

    pub async fn update_preferences(
        &self,
        minimize_to_tray_on_close: bool,
        launch_on_startup: bool,
        auto_switch_account_on_traffic_exhausted: bool,
    ) -> AppResult<AppSnapshotDto> {
        self.startup_controller
            .set_launch_on_startup(launch_on_startup)?;
        let launch_on_startup = self.startup_controller.is_enabled()?;
        let mut preferences = self.app_state_repo.load_preferences()?;
        preferences.minimize_to_tray_on_close = minimize_to_tray_on_close;
        preferences.launch_on_startup = launch_on_startup;
        preferences.auto_switch_account_on_traffic_exhausted =
            auto_switch_account_on_traffic_exhausted;
        self.app_state_repo.save_preferences(&preferences)?;
        self.refresh_runtime_from_disk()?;
        self.emit_state()
    }

    pub async fn refresh_dashboard(&self) -> AppResult<AppSnapshotDto> {
        self.run_refresh(true).await
    }

    pub async fn login_selected_account(&self) -> AppResult<AppSnapshotDto> {
        let _task_guard = self.network_task_lock.lock().await;
        {
            let mut state = self.state.write();
            if state.login_running {
                return Err(AppError::Conflict(
                    "HTTP 登录正在执行中，请稍等".to_string(),
                ));
            }
            state.login_running = true;
        }
        self.emit_task_started("login")?;
        let result = self.session_service.login_selected_account_inner().await;
        {
            let mut state = self.state.write();
            state.login_running = false;
        }
        self.emit_task_finished("login")?;
        self.emit_state()?;
        result?;
        self.run_refresh_inner(true).await
    }

    pub async fn logout_local_device(&self) -> AppResult<AppSnapshotDto> {
        let _task_guard = self.network_task_lock.lock().await;
        {
            let mut state = self.state.write();
            if state.logout_running || state.login_running {
                return Err(AppError::Conflict(
                    "当前有任务执行中，请稍后再试本机下线".to_string(),
                ));
            }
            state.logout_running = true;
        }
        self.emit_task_started("logout")?;
        let result = async {
            self.session_service.logout_local_device_inner().await?;
            self.run_refresh_inner(true).await.map(|_| ())
        }
        .await;
        {
            let mut state = self.state.write();
            state.logout_running = false;
        }
        self.emit_task_finished("logout")?;
        let settled_snapshot = self.emit_state()?;
        result?;
        Ok(settled_snapshot)
    }

    async fn run_refresh(&self, force: bool) -> AppResult<AppSnapshotDto> {
        let _task_guard = self.network_task_lock.lock().await;
        self.run_refresh_inner(force).await
    }

    async fn run_refresh_inner(&self, force: bool) -> AppResult<AppSnapshotDto> {
        if !force && self.is_quota_refresh_in_cooldown() {
            return self.emit_state();
        }
        {
            let mut state = self.state.write();
            if state.refresh_running {
                return self.build_snapshot();
            }
            state.refresh_running = true;
        }
        self.emit_task_started("refresh")?;
        let result = async {
            self.dashboard_refresh_service.refresh_accounts().await?;
            self.try_auto_switch().await
        }
        .await;
        {
            let mut state = self.state.write();
            state.refresh_running = false;
        }
        self.emit_task_finished("refresh")?;
        let settled_snapshot = self.emit_state()?;
        result?;
        Ok(settled_snapshot)
    }

    async fn refresh_dashboard_silently(&self) -> AppResult<()> {
        let Ok(_task_guard) = self.network_task_lock.try_lock() else {
            return Ok(());
        };
        let result = async {
            self.dashboard_refresh_service.refresh_accounts().await?;
            self.try_auto_switch().await
        }
        .await;
        self.emit_state()?;
        result?;
        Ok(())
    }

    async fn try_auto_switch(&self) -> AppResult<()> {
        let (enabled, store, snapshots, recent_ids) = {
            let state = self.state.read();
            (
                state.preferences.auto_switch_account_on_traffic_exhausted,
                state.account_store.clone(),
                state.snapshots.clone(),
                state.app_state.recent_account_ids.clone(),
            )
        };
        if !enabled {
            return Ok(());
        }
        let Some(target) = build_auto_switch_candidate(&store, &snapshots, &recent_ids) else {
            return Ok(());
        };
        let current = self.account_repo.get_selected_account(&store);
        if current.as_ref().is_some_and(|item| item.id == target.id) {
            return Ok(());
        }
        self.account_repo.select_account(&target.id)?;
        self.app_state_repo.mark_account_used(&target.id)?;
        self.refresh_runtime_from_disk()?;
        self.session_service.login_selected_account_inner().await?;
        let _ = current;
        Ok(())
    }

    fn start_background_refresh_loop(&self) {
        if self.background_refresh_started.swap(true, Ordering::SeqCst) {
            return;
        }
        let backend = self.clone();
        tokio::spawn(async move {
            sleep(std::time::Duration::from_secs(
                Self::BACKGROUND_REFRESH_INTERVAL_SECONDS,
            ))
            .await;
            loop {
                let _ = backend.refresh_dashboard_silently().await;
                sleep(std::time::Duration::from_secs(
                    Self::BACKGROUND_REFRESH_INTERVAL_SECONDS,
                ))
                .await;
            }
        });
    }

    fn refresh_runtime_from_disk(&self) -> AppResult<()> {
        refresh_runtime_from_disk(&self.state, &self.account_repo, &self.app_state_repo)
    }

    fn validate_account_input(
        &self,
        remark_name: &str,
        username: &str,
        password: &str,
    ) -> AppResult<()> {
        if remark_name.trim().is_empty() {
            return Err(AppError::Validation("备注名不能为空".to_string()));
        }
        if username.trim().is_empty() {
            return Err(AppError::Validation("账号不能为空".to_string()));
        }
        if password.trim().is_empty() {
            return Err(AppError::Validation("密码不能为空".to_string()));
        }
        Ok(())
    }

    fn validate_account_update_input(
        &self,
        remark_name: &str,
        username: &str,
        password: Option<&str>,
    ) -> AppResult<()> {
        if remark_name.trim().is_empty() {
            return Err(AppError::Validation("备注名不能为空".to_string()));
        }
        if username.trim().is_empty() {
            return Err(AppError::Validation("账号不能为空".to_string()));
        }
        if matches!(password, Some(value) if value.trim().is_empty()) {
            return Err(AppError::Validation(
                "密码为空就留空，别传一串空格".to_string(),
            ));
        }
        Ok(())
    }

    fn build_snapshot(&self) -> AppResult<AppSnapshotDto> {
        let state = self.state.read();
        Ok(build_app_snapshot(&state))
    }

    fn emit_state(&self) -> AppResult<AppSnapshotDto> {
        let snapshot = self.build_snapshot()?;
        self.event_sink.state_updated(&snapshot)?;
        if cfg!(target_os = "android") {
            let _ = write_android_keepalive_state(&self.app_data_dir, &snapshot);
        }
        Ok(snapshot)
    }

    fn emit_task_started(&self, task: &str) -> AppResult<()> {
        self.event_sink.task_started(task)
    }

    fn emit_task_finished(&self, task: &str) -> AppResult<()> {
        self.event_sink.task_finished(task)
    }

    fn is_quota_refresh_in_cooldown(&self) -> bool {
        let state = self.state.read();
        let Some(last_refresh_time) = state.app_state.last_quota_refresh_time else {
            return false;
        };
        Local::now()
            .signed_duration_since(last_refresh_time)
            .num_minutes()
            < Self::QUOTA_REFRESH_COOLDOWN_MINUTES
    }
}

#[cfg(test)]
#[path = "backend_tests.rs"]
mod tests;
