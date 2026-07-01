use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Local;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::application::dto::AppSnapshotDto;
use crate::application::error::{AppError, AppResult};
use crate::application::platform::{AppEventSink, RuntimePathProvider, StartupController};
use crate::application::runtime::{AppRuntimeState, SharedRuntimeState};
use crate::application::runtime_refresh::refresh_runtime_from_disk;
use crate::application::services::dashboard_refresh_service::DashboardRefreshService;
use crate::application::services::session_service::SessionService;
use crate::application::services::snapshot_mapper::{build_app_snapshot, restore_cached_snapshots};
use crate::domain::models::NetworkStatus;
use crate::domain::policies::traffic_math::build_auto_switch_candidate;
use crate::infrastructure::android_keepalive::write_android_keepalive_state;
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
        let vault: Arc<dyn CredentialVault> = Arc::new(SystemCredentialVault::initialize()?);
        let db = AppDatabase::open(&paths)?;
        let account_repo = AccountRepository::new(db.clone(), vault.clone());
        let app_state_repo = AppStateRepository::new(db.clone());
        let account_store = account_repo.ensure_store()?;
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
        let dashboard_refresh_service = DashboardRefreshService::new(
            runtime.clone(),
            account_repo.clone(),
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
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use chrono::Local;
    use tempfile::TempDir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::AppCore;
    use crate::application::dto::AppSnapshotDto;
    use crate::application::platform::{AppEventSink, NoopStartupController};
    use crate::application::runtime::{AppRuntimeState, SharedRuntimeState};
    use crate::application::services::dashboard_refresh_service::DashboardRefreshService;
    use crate::application::services::session_service::SessionService;
    use crate::domain::models::{CachedTrafficSnapshot, NetworkStatus};
    use crate::infrastructure::network::http_transport::HttpTransport;
    use crate::infrastructure::network::legacy_portal_auth_client::LegacyPortalAuthClient;
    use crate::infrastructure::network::legacy_portal_status_client::LegacyPortalStatusClient;
    use crate::infrastructure::network::network_status_service::NetworkStatusDetector;
    use crate::infrastructure::network::self_service_panel_client::SelfServicePanelClient;
    use crate::infrastructure::persistence::account_repository::AccountRepository;
    use crate::infrastructure::persistence::app_state_repository::AppStateRepository;
    use crate::infrastructure::persistence::database::AppDatabase;
    use crate::infrastructure::persistence::panel_session_repository::PanelSessionRepository;
    use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
    use crate::infrastructure::security::credential_vault::{
        CredentialVault, MemoryCredentialVault,
    };
    use crate::infrastructure::settings::AppSettings;

    #[derive(Clone)]
    struct FixedNetworkStatusDetector {
        ip: String,
    }

    impl NetworkStatusDetector for FixedNetworkStatusDetector {
        fn detect_network_status(&self) -> NetworkStatus {
            NetworkStatus {
                is_online: true,
                status_text: "IP 已识别".to_string(),
                ip: self.ip.clone(),
                checked_at: Local::now(),
            }
        }
    }

    #[derive(Default)]
    struct RecordingEventSink {
        events: Mutex<Vec<String>>,
    }

    impl RecordingEventSink {
        fn events(&self) -> Vec<String> {
            self.events.lock().expect("events lock").clone()
        }
    }

    impl AppEventSink for RecordingEventSink {
        fn state_updated(&self, snapshot: &AppSnapshotDto) -> crate::application::AppResult<()> {
            let running = snapshot.login_state.running || snapshot.refresh_state.running;
            self.events
                .lock()
                .expect("events lock")
                .push(format!("state:running={running}"));
            Ok(())
        }

        fn task_started(&self, task: &str) -> crate::application::AppResult<()> {
            self.events
                .lock()
                .expect("events lock")
                .push(format!("start:{task}"));
            Ok(())
        }

        fn task_finished(&self, task: &str) -> crate::application::AppResult<()> {
            self.events
                .lock()
                .expect("events lock")
                .push(format!("finish:{task}"));
            Ok(())
        }
    }

    fn build_test_core(
        settings: AppSettings,
        local_ip: &str,
    ) -> (AppCore, TempDir, Arc<RecordingEventSink>) {
        let root = tempfile::tempdir().expect("create temp dir");
        let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
        let db = AppDatabase::open(&paths).expect("open db");
        let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
        let account_repo = AccountRepository::new(db.clone(), vault);
        let app_state_repo = AppStateRepository::new(db.clone());
        let account_store = account_repo.ensure_store().expect("ensure store");
        let app_state = app_state_repo.load_state().expect("load state");
        let preferences = app_state_repo.load_preferences().expect("load preferences");
        let panel_session_repo = PanelSessionRepository::new(db);
        let auth_transport = HttpTransport::new(settings.clone()).expect("auth transport");
        let legacy_portal_transport =
            HttpTransport::new(settings.clone()).expect("status transport");
        let panel_transport = HttpTransport::new(settings.clone()).expect("panel transport");
        let auth_client = LegacyPortalAuthClient::new(settings.clone(), auth_transport);
        let portal_status_client =
            LegacyPortalStatusClient::new(settings.clone(), legacy_portal_transport);
        let panel_client =
            SelfServicePanelClient::new(settings, panel_transport, panel_session_repo);
        let network_status_service: Arc<dyn NetworkStatusDetector> =
            Arc::new(FixedNetworkStatusDetector {
                ip: local_ip.to_string(),
            });
        let event_sink = Arc::new(RecordingEventSink::default());
        let runtime = SharedRuntimeState::new(AppRuntimeState {
            account_store: account_store.clone(),
            app_state: app_state.clone(),
            preferences,
            network: NetworkStatus::default(),
            snapshots: Default::default(),
            current_online_account_id: account_store.current_online_account_id.clone(),
            login_running: false,
            refresh_running: false,
            logout_running: false,
        });
        let session_service = SessionService::new(
            runtime.clone(),
            account_repo.clone(),
            app_state_repo.clone(),
            auth_client,
            portal_status_client.clone(),
            network_status_service.clone(),
        );
        let dashboard_refresh_service = DashboardRefreshService::new(
            runtime.clone(),
            account_repo.clone(),
            app_state_repo.clone(),
            portal_status_client,
            panel_client,
            network_status_service,
            event_sink.clone(),
        );
        let core = AppCore {
            state: runtime,
            account_repo,
            app_state_repo,
            session_service,
            dashboard_refresh_service,
            network_task_lock: Arc::new(tokio::sync::Mutex::new(())),
            background_refresh_started: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            app_data_dir: paths.app_data_dir().to_path_buf(),
            event_sink: event_sink.clone(),
            startup_controller: Arc::new(NoopStartupController),
        };
        (core, root, event_sink)
    }

    fn settings_for(server: &MockServer) -> AppSettings {
        AppSettings {
            portal_url: format!("{}/srun_portal_pc.php?ac_id=1&", server.uri()),
            traffic_portal_url: format!("{}/home", server.uri()),
            ..Default::default()
        }
    }

    fn success_page(ip: &str, username: &str) -> String {
        format!("当前的ip：{ip}\n上网用户：{username}\n已用流量：1.00G\n计费方式：flow")
    }

    fn panel_home_html(ip: &str) -> String {
        format!(
            r#"
            <table>
              <tr><th>产品名称</th><th>计费策略</th><th>已用流量</th><th>产品余额</th></tr>
              <tr><td>校园网</td><td>免费70GB</td><td>1.00GB</td><td>69.00GB</td></tr>
            </table>
            <tr data-key="device-a">
              <td data-col-seq="1">{ip}</td>
              <td><a href="/home/delete?id=device-a">下线</a></td>
            </tr>
            "#
        )
    }

    #[tokio::test]
    async fn login_switches_online_ip_with_login_post_without_logout() {
        let server = MockServer::start().await;
        let local_ip = "10.151.119.57";
        let success_count = Arc::new(AtomicUsize::new(0));
        let success_count_for_mock = success_count.clone();
        Mock::given(method("GET"))
            .and(path("/srun_portal_pc_success.php"))
            .respond_with(move |_request: &wiremock::Request| {
                let count = success_count_for_mock.fetch_add(1, Ordering::SeqCst);
                let username = if count == 0 { "20260001" } else { "20260002" };
                ResponseTemplate::new(200).set_body_string(success_page(local_ip, username))
            })
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/srun_portal_pc.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("login_ok,ok"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/include/auth_action.php"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(format!("1073741824,60,0.00,aa:bb:cc:dd:ee:ff,0,{local_ip}")),
            )
            .mount(&server)
            .await;

        let (core, _root, event_sink) = build_test_core(settings_for(&server), local_ip);
        let first_snapshot = core
            .add_account("旧号".to_string(), "20260001".to_string(), "p1".to_string())
            .await
            .expect("add first");
        let second_snapshot = core
            .add_account(
                "目标号".to_string(),
                "20260002".to_string(),
                "p2".to_string(),
            )
            .await
            .expect("add second");
        let first_id = first_snapshot
            .accounts
            .iter()
            .find(|account| account.username == "20260001")
            .expect("first account")
            .id
            .clone();
        let second_id = second_snapshot
            .accounts
            .iter()
            .find(|account| account.username == "20260002")
            .expect("second account")
            .id
            .clone();
        core.select_account(second_id.clone())
            .await
            .expect("select second");

        let snapshot = core.login_selected_account().await.expect("login selected");

        assert_eq!(snapshot.current_online_account_id, second_id);
        assert_ne!(snapshot.current_online_account_id, first_id);
        let requests = server.received_requests().await.unwrap_or_default();
        let bodies = requests
            .iter()
            .map(|request| String::from_utf8_lossy(&request.body).to_string())
            .collect::<Vec<_>>();
        assert!(bodies
            .iter()
            .any(|body| body.contains("action=login") && body.contains("username=20260002")));
        assert!(!bodies.iter().any(|body| body.contains("action=logout")));
        let events = event_sink.events();
        assert!(events.contains(&"start:login".to_string()));
        assert!(events.contains(&"finish:login".to_string()));
        assert!(events.contains(&"start:refresh".to_string()));
        assert!(events.contains(&"finish:refresh".to_string()));
    }

    #[tokio::test]
    async fn login_switches_when_current_online_account_is_not_in_pool() {
        let server = MockServer::start().await;
        let local_ip = "10.151.119.57";
        let success_count = Arc::new(AtomicUsize::new(0));
        let success_count_for_mock = success_count.clone();
        Mock::given(method("GET"))
            .and(path("/srun_portal_pc_success.php"))
            .respond_with(move |_request: &wiremock::Request| {
                let count = success_count_for_mock.fetch_add(1, Ordering::SeqCst);
                let username = if count == 0 {
                    "external-user"
                } else {
                    "20260002"
                };
                ResponseTemplate::new(200).set_body_string(success_page(local_ip, username))
            })
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/srun_portal_pc.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("login_ok,ok"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/include/auth_action.php"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(format!("1073741824,60,0.00,aa:bb:cc:dd:ee:ff,0,{local_ip}")),
            )
            .mount(&server)
            .await;

        let (core, _root, _event_sink) = build_test_core(settings_for(&server), local_ip);
        let account_snapshot = core
            .add_account(
                "目标号".to_string(),
                "20260002".to_string(),
                "p2".to_string(),
            )
            .await
            .expect("add target");
        let target_id = account_snapshot.selected_account_id;

        let snapshot = core.login_selected_account().await.expect("login selected");

        assert_eq!(snapshot.current_online_account_id, target_id);
        let requests = server.received_requests().await.unwrap_or_default();
        let bodies = requests
            .iter()
            .map(|request| String::from_utf8_lossy(&request.body).to_string())
            .collect::<Vec<_>>();
        assert!(bodies
            .iter()
            .any(|body| body.contains("action=login") && body.contains("username=20260002")));
        assert!(!bodies.iter().any(|body| body.contains("action=logout")));
    }

    #[tokio::test]
    async fn refresh_uses_success_page_and_sso_panel_home_for_current_account() {
        let server = MockServer::start().await;
        let local_ip = "10.151.119.57";
        Mock::given(method("GET"))
            .and(path("/include/auth_action.php"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(format!("1073741824,60,0.00,aa:bb:cc:dd:ee:ff,0,{local_ip}")),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/srun_portal_pc_success.php"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(success_page(local_ip, "20260001")),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/site/sso"))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("set-cookie", "PHPSESSID_8800=abc; path=/; HttpOnly")
                    .insert_header("location", "/home"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/home"))
            .respond_with(ResponseTemplate::new(200).set_body_string(panel_home_html(local_ip)))
            .mount(&server)
            .await;

        let (core, _root, _event_sink) = build_test_core(settings_for(&server), local_ip);
        let snapshot = core
            .add_account("主号".to_string(), "20260001".to_string(), "p1".to_string())
            .await
            .expect("add account");

        let refreshed = core.refresh_dashboard().await.expect("refresh dashboard");

        assert_eq!(
            refreshed.current_online_account_id,
            snapshot.selected_account_id
        );
        let account = refreshed.accounts.first().expect("account");
        let traffic = account.snapshot.as_ref().expect("traffic snapshot");
        assert_eq!(traffic.package_text, "校园网");
        assert_eq!(traffic.online_device_count_text, "1");
        assert!(account.can_logout_local_device);
    }

    #[tokio::test]
    async fn silent_refresh_does_not_emit_refresh_running_state() {
        let server = MockServer::start().await;
        let local_ip = "10.151.119.57";
        Mock::given(method("GET"))
            .and(path("/include/auth_action.php"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(format!("1073741824,60,0.00,aa:bb:cc:dd:ee:ff,0,{local_ip}")),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/srun_portal_pc_success.php"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(success_page(local_ip, "20260001")),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/site/sso"))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("set-cookie", "PHPSESSID_8800=abc; path=/; HttpOnly")
                    .insert_header("location", "/home"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/home"))
            .respond_with(ResponseTemplate::new(200).set_body_string(panel_home_html(local_ip)))
            .mount(&server)
            .await;

        let (core, _root, event_sink) = build_test_core(settings_for(&server), local_ip);
        core.add_account("主号".to_string(), "20260001".to_string(), "p1".to_string())
            .await
            .expect("add account");

        core.refresh_dashboard_silently()
            .await
            .expect("silent refresh");

        let events = event_sink.events();
        assert!(!events.iter().any(|event| event == "start:refresh"));
        assert!(!events.iter().any(|event| event == "finish:refresh"));
        assert!(events.iter().any(|event| event == "state:running=false"));
    }

    #[tokio::test]
    async fn silent_refresh_auto_switches_to_most_recent_previous_account() {
        let server = MockServer::start().await;
        let local_ip = "10.151.119.57";
        let success_count = Arc::new(AtomicUsize::new(0));
        let success_count_for_mock = success_count.clone();
        Mock::given(method("GET"))
            .and(path("/srun_portal_pc_success.php"))
            .respond_with(move |_request: &wiremock::Request| {
                let count = success_count_for_mock.fetch_add(1, Ordering::SeqCst);
                let username = if count < 2 { "20260002" } else { "20260001" };
                ResponseTemplate::new(200).set_body_string(success_page(local_ip, username))
            })
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/include/auth_action.php"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(format!("1073741824,60,0.00,aa:bb:cc:dd:ee:ff,0,{local_ip}")),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/site/sso"))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("set-cookie", "PHPSESSID_8800=abc; path=/; HttpOnly")
                    .insert_header("location", "/home"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/home"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                r#"
                    <table>
                      <tr><th>产品名称</th><th>计费策略</th><th>已用流量</th><th>产品余额</th></tr>
                      <tr><td>校园网</td><td>免费70GB</td><td>70.00GB</td><td>70.00GB</td></tr>
                    </table>
                    <tr data-key="device-a">
                      <td data-col-seq="1">{local_ip}</td>
                      <td><a href="/home/delete?id=device-a">下线</a></td>
                    </tr>
                    "#
            )))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/srun_portal_pc.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("login_ok,ok"))
            .mount(&server)
            .await;

        let (core, _root, _event_sink) = build_test_core(settings_for(&server), local_ip);
        let first_snapshot = core
            .add_account(
                "上一个号".to_string(),
                "20260001".to_string(),
                "p1".to_string(),
            )
            .await
            .expect("add first");
        let second_snapshot = core
            .add_account(
                "当前号".to_string(),
                "20260002".to_string(),
                "p2".to_string(),
            )
            .await
            .expect("add second");
        let first_id = first_snapshot
            .accounts
            .iter()
            .find(|account| account.username == "20260001")
            .expect("first account")
            .id
            .clone();
        let second_id = second_snapshot
            .accounts
            .iter()
            .find(|account| account.username == "20260002")
            .expect("second account")
            .id
            .clone();

        let mut preferences = core
            .app_state_repo
            .load_preferences()
            .expect("load preferences");
        preferences.auto_switch_account_on_traffic_exhausted = true;
        core.app_state_repo
            .save_preferences(&preferences)
            .expect("save preferences");
        core.refresh_runtime_from_disk().expect("refresh runtime");

        core.select_account(first_id.clone())
            .await
            .expect("select first");
        core.select_account(second_id.clone())
            .await
            .expect("reselect second");
        let mut store = core.account_repo.load_store().expect("load store");
        store.cached_traffic_snapshots.insert(
            first_id.clone(),
            CachedTrafficSnapshot {
                used_traffic_text: "10.00GB".to_string(),
                product_balance_text: "70.00GB".to_string(),
                status_text: "已同步".to_string(),
                detail_text: "测试缓存快照".to_string(),
                progress_percent: Some(14.3),
                ..Default::default()
            },
        );
        core.account_repo.save_store(&store).expect("save store");
        core.refresh_runtime_from_disk().expect("refresh runtime");

        core.refresh_dashboard_silently()
            .await
            .expect("silent refresh");

        let snapshot = core.get_snapshot().expect("snapshot");
        assert_eq!(snapshot.selected_account_id, first_id);
        assert_eq!(snapshot.current_online_account_id, first_id);
        let requests = server.received_requests().await.unwrap_or_default();
        let bodies = requests
            .iter()
            .map(|request| String::from_utf8_lossy(&request.body).to_string())
            .collect::<Vec<_>>();
        assert!(bodies
            .iter()
            .any(|body| body.contains("action=login") && body.contains("username=20260001")));
    }

    #[tokio::test]
    async fn logout_local_device_posts_success_page_logout_and_clears_current_account() {
        let server = MockServer::start().await;
        let local_ip = "10.151.119.57";
        Mock::given(method("GET"))
            .and(path("/srun_portal_pc.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string(format!(
                r#"
                {}
                <form>
                  <input name="action" value="auto_logout">
                  <input name="ac_id" value="1">
                  <input name="info" value="">
                  <input name="user_ip" value="{local_ip}">
                  <input name="username" value="20260001">
                </form>
                "#,
                success_page(local_ip, "20260001")
            )))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/srun_portal_pc_success.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("网络已断开"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/include/auth_action.php"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not_online"))
            .mount(&server)
            .await;

        let (core, _root, event_sink) = build_test_core(settings_for(&server), local_ip);
        let snapshot = core
            .add_account("主号".to_string(), "20260001".to_string(), "p1".to_string())
            .await
            .expect("add account");
        let mut store = core.account_repo.load_store().expect("load store");
        store.current_online_account_id = snapshot.selected_account_id.clone();
        core.account_repo.save_store(&store).expect("save store");
        core.refresh_runtime_from_disk().expect("refresh runtime");

        let snapshot = core
            .logout_local_device()
            .await
            .expect("logout local device");

        assert_eq!(snapshot.current_online_account_id, "");
        let requests = server.received_requests().await.unwrap_or_default();
        let bodies = requests
            .iter()
            .map(|request| String::from_utf8_lossy(&request.body).to_string())
            .collect::<Vec<_>>();
        assert!(bodies
            .iter()
            .any(|body| body.contains("action=auto_logout")));
        let events = event_sink.events();
        assert!(events.contains(&"start:logout".to_string()));
        assert!(events.contains(&"finish:logout".to_string()));
    }

    #[tokio::test]
    async fn login_failure_emits_settled_state() {
        let server = MockServer::start().await;
        let local_ip = "10.151.119.57";

        let (core, _root, event_sink) = build_test_core(settings_for(&server), local_ip);

        let result = core.login_selected_account().await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), "VALIDATION_ERROR");
        let events = event_sink.events();
        assert!(events.contains(&"start:login".to_string()));
        assert!(events.contains(&"finish:login".to_string()));
        assert_eq!(events.last(), Some(&"state:running=false".to_string()));
    }
}
