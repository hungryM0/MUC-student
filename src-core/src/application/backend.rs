use std::sync::Arc;

use chrono::Local;

use crate::application::dto::AppSnapshotDto;
use crate::application::error::{AppError, AppResult};
use crate::application::platform::{AppEventSink, RuntimePathProvider, StartupController};
use crate::application::runtime::{AppRuntimeState, SharedRuntimeState};
use crate::application::services::account_traffic_service::AccountTrafficService;
use crate::application::services::dashboard_refresh_service::{
    DashboardRefreshDependencies, DashboardRefreshService,
};
use crate::application::services::portal_snapshot_service::PortalSnapshotService;
use crate::application::services::session_service::SessionService;
use crate::application::services::snapshot_mapper::{build_app_snapshot, restore_cached_snapshots};
use crate::domain::models::NetworkStatus;
use crate::domain::policies::traffic_math::build_auto_switch_candidate;
use crate::infrastructure::network::{
    http_transport::HttpTransport, legacy_portal_auth_client::LegacyPortalAuthClient,
    legacy_portal_status_client::LegacyPortalStatusClient,
    network_status_service::NetworkStatusService,
    self_service_panel_client::SelfServicePanelClient,
};
use crate::infrastructure::persistence::account_repository::AccountRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;
use crate::infrastructure::persistence::migration::MigrationService;
use crate::infrastructure::persistence::panel_session_repository::PanelSessionRepository;
use crate::infrastructure::persistence::runtime_paths::{resolve_default_paths, RuntimePaths};
use crate::infrastructure::security::credential_vault::{CredentialVault, WindowsCredentialVault};
use crate::infrastructure::settings::AppSettings;

#[derive(Clone)]
pub struct AppCore {
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    app_state_repo: AppStateRepository,
    session_service: SessionService,
    dashboard_refresh_service: DashboardRefreshService,
    event_sink: Arc<dyn AppEventSink>,
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
            path_provider.legacy_root()?,
        )?;
        Self::build_with_paths(paths, startup_controller, event_sink)
    }

    pub fn build_with_paths(
        paths: RuntimePaths,
        _startup_controller: Arc<dyn StartupController>,
        event_sink: Arc<dyn AppEventSink>,
    ) -> AppResult<Self> {
        let settings = AppSettings::default();
        let vault: Arc<dyn CredentialVault> = Arc::new(WindowsCredentialVault::initialize()?);
        let account_repo = AccountRepository::new(paths.clone(), vault.clone());
        let app_state_repo = AppStateRepository::new(paths.clone());
        let migration = MigrationService::new(
            paths.clone(),
            vault.clone(),
            account_repo.clone(),
            app_state_repo.clone(),
        );
        let _migrated = migration.migrate_if_needed()?;
        let account_store = account_repo.ensure_store()?;
        let app_state = app_state_repo.load_state()?;
        let preferences = app_state_repo.load_preferences()?;
        let panel_session_repo = PanelSessionRepository::new(paths.clone());

        let auth_transport = HttpTransport::new(settings.clone());
        let legacy_portal_transport = HttpTransport::new(settings.clone());
        let panel_transport = HttpTransport::new(settings.clone());
        let auth_client = LegacyPortalAuthClient::new(settings.clone(), auth_transport);
        let portal_status_client =
            LegacyPortalStatusClient::new(settings.clone(), legacy_portal_transport);
        let panel_client =
            SelfServicePanelClient::new(settings.clone(), panel_transport, panel_session_repo);
        let traffic_service = AccountTrafficService::new(panel_client.clone());
        let portal_snapshot_service = PortalSnapshotService::new(
            account_repo.clone(),
            auth_client.clone(),
            portal_status_client.clone(),
        );
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
                traffic_service,
                portal_snapshot_service,
                network_status_service,
                event_sink: event_sink.clone(),
            });
        let backend = Self {
            state: runtime,
            account_repo,
            app_state_repo,
            session_service,
            dashboard_refresh_service,
            event_sink,
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

    pub async fn refresh_dashboard(&self) -> AppResult<AppSnapshotDto> {
        self.run_refresh(true).await
    }

    pub async fn login_selected_account(&self) -> AppResult<AppSnapshotDto> {
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
            self.run_refresh(true).await.map(|_| ())
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
