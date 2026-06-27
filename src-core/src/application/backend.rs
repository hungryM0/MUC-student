use std::sync::Arc;

use chrono::Local;
use tokio::sync::Mutex;

use crate::application::dto::AppSnapshotDto;
use crate::application::error::{AppError, AppResult};
use crate::application::platform::{AppEventSink, RuntimePathProvider, StartupController};
use crate::application::runtime::{AppRuntimeState, SharedRuntimeState};
use crate::application::services::account_traffic_service::AccountTrafficService;
use crate::application::services::dashboard_refresh_service::{
    DashboardRefreshDependencies, DashboardRefreshService,
};
use crate::application::services::session_service::SessionService;
use crate::application::services::snapshot_mapper::{build_app_snapshot, restore_cached_snapshots};
use crate::domain::models::{NetworkStatus, PortalAccount};
use crate::domain::policies::traffic_math::build_auto_switch_candidate;
use crate::infrastructure::network::{
    http_transport::HttpTransport, legacy_portal_auth_client::LegacyPortalAuthClient,
    legacy_portal_status_client::LegacyPortalStatusClient,
    network_status_service::NetworkStatusService,
    self_service_panel_client::SelfServicePanelClient,
};
use crate::infrastructure::persistence::account_repository::AccountRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;
use crate::infrastructure::persistence::database::AppDatabase;
use crate::infrastructure::persistence::panel_session_repository::PanelSessionRepository;
use crate::infrastructure::persistence::runtime_paths::{resolve_default_paths, RuntimePaths};
use crate::infrastructure::security::credential_vault::{CredentialVault, WindowsCredentialVault};
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct AppCore {
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    app_state_repo: AppStateRepository,
    account_traffic_service: AccountTrafficService,
    session_service: SessionService,
    dashboard_refresh_service: DashboardRefreshService,
    network_task_lock: Arc<Mutex<()>>,
    event_sink: Arc<dyn AppEventSink>,
    startup_controller: Arc<dyn StartupController>,
}

impl AppCore {
    const QUOTA_REFRESH_COOLDOWN_MINUTES: i64 = 30;

    pub fn build(
        path_provider: Arc<dyn RuntimePathProvider>,
        startup_controller: Arc<dyn StartupController>,
        event_sink: Arc<dyn AppEventSink>,
    ) -> AppResult<Self> {
        let paths = RuntimePaths::new(
            path_provider.app_data_dir()?,
            path_provider.resource_base_dir()?,
        )?;
        Self::build_with_paths(paths, startup_controller, event_sink)
    }

    pub fn build_with_paths(
        paths: RuntimePaths,
        startup_controller: Arc<dyn StartupController>,
        event_sink: Arc<dyn AppEventSink>,
    ) -> AppResult<Self> {
        let settings = AppSettings::default();
        let vault: Arc<dyn CredentialVault> = Arc::new(WindowsCredentialVault::initialize()?);
        let db = AppDatabase::open(&paths)?;
        let account_repo = AccountRepository::new(db.clone(), vault.clone());
        let app_state_repo = AppStateRepository::new(db.clone());
        let account_store = account_repo.ensure_store()?;
        let app_state = app_state_repo.load_state()?;
        let mut preferences = app_state_repo.load_preferences()?;
        preferences.launch_on_startup = startup_controller.is_enabled()?;
        let panel_session_repo = PanelSessionRepository::new(db.clone());

        let auth_transport = HttpTransport::new(settings.clone())?;
        let legacy_portal_transport = HttpTransport::new(settings.clone())?;
        let panel_transport = HttpTransport::new(settings.clone())?;
        let auth_client = LegacyPortalAuthClient::new(settings.clone(), auth_transport);
        let portal_status_client =
            LegacyPortalStatusClient::new(settings.clone(), legacy_portal_transport);
        let panel_client =
            SelfServicePanelClient::new(settings.clone(), panel_transport, panel_session_repo);
        let traffic_service = AccountTrafficService::new(panel_client.clone());
        let network_status_service = Arc::new(NetworkStatusService::new(settings));
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
            app_state_repo.clone(),
            auth_client.clone(),
            portal_status_client.clone(),
            network_status_service.clone(),
        );
        let dashboard_refresh_service =
            DashboardRefreshService::new(DashboardRefreshDependencies {
                state: runtime.clone(),
                account_repo: account_repo.clone(),
                app_state_repo: app_state_repo.clone(),
                portal_status_client,
                panel_client: panel_client.clone(),
                network_status_service,
                event_sink: event_sink.clone(),
            });
        let backend = Self {
            state: runtime,
            account_repo,
            app_state_repo,
            account_traffic_service: traffic_service,
            session_service,
            dashboard_refresh_service,
            network_task_lock: Arc::new(Mutex::new(())),
            event_sink,
            startup_controller,
        };
        Ok(backend)
    }

    pub fn build_default(
        startup_controller: Arc<dyn StartupController>,
        event_sink: Arc<dyn AppEventSink>,
    ) -> AppResult<Self> {
        Self::build_with_paths(resolve_default_paths()?, startup_controller, event_sink)
    }

    pub async fn bootstrap_app(&self) -> AppResult<AppSnapshotDto> {
        self.refresh_runtime_from_disk()?;
        let snapshot = self.emit_state()?;
        let backend = self.clone();
        tokio::spawn(async move {
            if backend.run_refresh(false).await.is_err() {
                let _ = backend.emit_state();
            }
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
        let validation_warning = self
            .validate_saved_account_credentials(&account.id)
            .await
            .err()
            .map(|error| error.to_string());
        let mut snapshot = self.emit_state()?;
        if let Some(message) = validation_warning {
            snapshot.login_state.message = message;
        }
        Ok(snapshot)
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
        let validation_warning =
            if self.should_validate_updated_account(&account, password.as_deref())? {
                self.validate_saved_account_credentials(&account.id)
                    .await
                    .err()
                    .map(|error| error.to_string())
            } else {
                None
            };
        let mut snapshot = self.emit_state()?;
        if let Some(message) = validation_warning {
            snapshot.login_state.message = message;
        }
        Ok(snapshot)
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

    pub async fn update_preferences(
        &self,
        minimize_to_tray_on_close: bool,
        launch_on_startup: bool,
        auto_switch_account_on_traffic_exhausted: bool,
    ) -> AppResult<AppSnapshotDto> {
        self.startup_controller
            .set_launch_on_startup(launch_on_startup)?;
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
        result?;
        self.emit_state()
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
        result?;
        self.emit_state()
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
        result?;
        self.emit_state()
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

    fn refresh_runtime_from_disk(&self) -> AppResult<()> {
        let account_store = self.account_repo.load_store()?;
        let app_state = self.app_state_repo.load_state()?;
        let preferences = self.app_state_repo.load_preferences()?;
        let valid_ids = account_store
            .accounts
            .iter()
            .map(|account| account.id.clone())
            .collect::<std::collections::HashSet<_>>();
        let mut state = self.state.write();
        state.current_online_account_id = account_store.current_online_account_id.clone();
        state.snapshots.retain(|id, _| valid_ids.contains(id));
        for (id, snapshot) in restore_cached_snapshots(&account_store.cached_traffic_snapshots) {
            state.snapshots.entry(id).or_insert(snapshot);
        }
        state.account_store = account_store;
        state.app_state = app_state;
        state.preferences = preferences;
        Ok(())
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

    async fn validate_saved_account_credentials(&self, account_id: &str) -> AppResult<()> {
        let store = self.account_repo.load_store()?;
        let account = self
            .account_repo
            .get_account_by_id(&store, account_id)
            .ok_or_else(|| AppError::NotFound("刚保存的账号找不到了".to_string()))?;
        let account = self.account_repo.load_account_with_password(&account)?;
        self.account_traffic_service
            .fetch_balance(&account, None)
            .await
            .map(|_| ())
            .map_err(|error| {
                AppError::Validation(format!(
                    "账号已保存，但校验失败：{}。你仍然可以稍后手动登录或刷新再看。",
                    error
                ))
            })
    }

    fn should_validate_updated_account(
        &self,
        account: &PortalAccount,
        password: Option<&str>,
    ) -> AppResult<bool> {
        if matches!(password, Some(value) if !value.trim().is_empty()) {
            return Ok(true);
        }
        let state = self.state.read();
        let previous = state
            .account_store
            .accounts
            .iter()
            .find(|item| item.id == account.id)
            .ok_or_else(|| AppError::NotFound("找不到要编辑的账号".to_string()))?;
        Ok(previous.username != account.username)
    }

    fn build_snapshot(&self) -> AppResult<AppSnapshotDto> {
        let state = self.state.read();
        Ok(build_app_snapshot(&state))
    }

    fn emit_state(&self) -> AppResult<AppSnapshotDto> {
        let snapshot = self.build_snapshot()?;
        self.event_sink.state_updated(&snapshot)?;
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
