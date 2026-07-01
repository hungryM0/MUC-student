use std::collections::HashMap;
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
use crate::infrastructure::persistence::account_snapshot_repository::AccountSnapshotRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;
use crate::infrastructure::persistence::database::AppDatabase;
use crate::infrastructure::persistence::panel_session_repository::PanelSessionRepository;
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
use crate::infrastructure::security::credential_vault::{CredentialVault, MemoryCredentialVault};
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
) -> (
    AppCore,
    TempDir,
    Arc<RecordingEventSink>,
    PanelSessionRepository,
) {
    let root = tempfile::tempdir().expect("create temp dir");
    let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
    let db = AppDatabase::open(&paths).expect("open db");
    let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
    let account_repo = AccountRepository::new(db.clone(), vault);
    let snapshot_repo = AccountSnapshotRepository::new(db.clone());
    let app_state_repo = AppStateRepository::new(db.clone());
    let account_store = account_repo.ensure_store().expect("ensure store");
    let app_state = app_state_repo.load_state().expect("load state");
    let preferences = app_state_repo.load_preferences().expect("load preferences");
    let panel_session_repo = PanelSessionRepository::new(db);
    let auth_transport = HttpTransport::new(settings.clone()).expect("auth transport");
    let legacy_portal_transport = HttpTransport::new(settings.clone()).expect("status transport");
    let panel_transport = HttpTransport::new(settings.clone()).expect("panel transport");
    let auth_client = LegacyPortalAuthClient::new(settings.clone(), auth_transport);
    let portal_status_client =
        LegacyPortalStatusClient::new(settings.clone(), legacy_portal_transport);
    let panel_client =
        SelfServicePanelClient::new(settings, panel_transport, panel_session_repo.clone());
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
        snapshot_repo.clone(),
        app_state_repo.clone(),
        auth_client,
        portal_status_client.clone(),
        network_status_service.clone(),
    );
    let dashboard_refresh_service = DashboardRefreshService::new(
        runtime.clone(),
        account_repo.clone(),
        snapshot_repo,
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
    (core, root, event_sink, panel_session_repo)
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

    let (core, _root, event_sink, _panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);
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
async fn login_waits_for_success_page_after_portal_accepts_switch() {
    let server = MockServer::start().await;
    let local_ip = "10.151.119.57";
    let success_count = Arc::new(AtomicUsize::new(0));
    let success_count_for_mock = success_count.clone();
    Mock::given(method("GET"))
        .and(path("/srun_portal_pc_success.php"))
        .respond_with(move |_request: &wiremock::Request| {
            let count = success_count_for_mock.fetch_add(1, Ordering::SeqCst);
            let username = if count < 4 { "20260001" } else { "20260002" };
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

    let (core, _root, _event_sink, _panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);
    core.add_account("旧号".to_string(), "20260001".to_string(), "p1".to_string())
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
    assert_eq!(snapshot.login_state.result_text, "成功");
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
async fn login_treats_already_online_response_as_success_when_success_page_matches_target() {
    let server = MockServer::start().await;
    let local_ip = "10.151.119.57";
    let success_count = Arc::new(AtomicUsize::new(0));
    let success_count_for_mock = success_count.clone();
    Mock::given(method("GET"))
        .and(path("/srun_portal_pc_success.php"))
        .respond_with(move |_request: &wiremock::Request| {
            let count = success_count_for_mock.fetch_add(1, Ordering::SeqCst);
            let username = if count < 3 { "20260001" } else { "20260002" };
            ResponseTemplate::new(200).set_body_string(success_page(local_ip, username))
        })
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/srun_portal_pc.php"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("IP has been online, please logout."),
        )
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
        .respond_with(ResponseTemplate::new(200).set_body_string(panel_home_html(local_ip)))
        .mount(&server)
        .await;

    let (core, _root, _event_sink, _panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);
    core.add_account("旧号".to_string(), "20260001".to_string(), "p1".to_string())
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
    assert_eq!(snapshot.login_state.result_text, "成功");
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

    let (core, _root, _event_sink, _panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);
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

    let (core, _root, _event_sink, _panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);
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

    let (core, _root, event_sink, _panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);
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

    let (core, _root, _event_sink, _panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);
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
async fn login_refresh_prefers_success_page_account_over_stale_cached_panel_session() {
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
    Mock::given(method("GET"))
        .and(path("/include/auth_action.php"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(format!("1073741824,60,0.00,aa:bb:cc:dd:ee:ff,0,{local_ip}")),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/srun_portal_pc.php"))
        .respond_with(ResponseTemplate::new(200).set_body_string("login_ok,ok"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/site/sso"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("set-cookie", "PHPSESSID_8800=fresh; path=/; HttpOnly")
                .insert_header("location", "/home"),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/home"))
        .respond_with(move |request: &wiremock::Request| {
            let cookie = request
                .headers
                .get("cookie")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default();
            let body = if cookie.contains("stale-session") {
                format!(
                    r#"
                    <table>
                      <tr><th>产品名称</th><th>计费策略</th><th>已用流量</th><th>产品余额</th></tr>
                      <tr><td>校园网</td><td>免费70GB</td><td>70.00GB</td><td>70.00GB</td></tr>
                    </table>
                    <tr data-key="device-stale">
                      <td data-col-seq="1">{local_ip}</td>
                      <td><a href="/home/delete?id=device-stale">下线</a></td>
                    </tr>
                    "#
                )
            } else {
                panel_home_html(local_ip)
            };
            ResponseTemplate::new(200).set_body_string(body)
        })
        .mount(&server)
        .await;

    let (core, _root, _event_sink, panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);
    let first_snapshot = core
        .add_account(
            "上个月满了".to_string(),
            "20260001".to_string(),
            "p1".to_string(),
        )
        .await
        .expect("add first");
    let second_snapshot = core
        .add_account(
            "这个月正常".to_string(),
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

    panel_session_repo
        .save_session(
            &second_id,
            &HashMap::from([("PHPSESSID_8800".to_string(), "stale-session".to_string())]),
        )
        .expect("insert stale panel session");

    core.select_account(second_id.clone())
        .await
        .expect("select second");

    let snapshot = core.login_selected_account().await.expect("login selected");

    assert_eq!(snapshot.current_online_account_id, second_id);
    assert_eq!(snapshot.selected_account_id, second_id);
    let current_account = snapshot
        .accounts
        .iter()
        .find(|account| account.id == second_id)
        .expect("current account");
    let traffic = current_account.snapshot.as_ref().expect("traffic snapshot");
    assert_eq!(traffic.used_traffic_text, "1.00GB");
    assert_eq!(traffic.progress_percent, Some(1.4));
    assert_ne!(snapshot.current_online_account_id, first_id);
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

    let (core, _root, event_sink, _panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);
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

    let (core, _root, event_sink, _panel_session_repo) =
        build_test_core(settings_for(&server), local_ip);

    let result = core.login_selected_account().await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), "VALIDATION_ERROR");
    let events = event_sink.events();
    assert!(events.contains(&"start:login".to_string()));
    assert!(events.contains(&"finish:login".to_string()));
    assert_eq!(events.last(), Some(&"state:running=false".to_string()));
}
