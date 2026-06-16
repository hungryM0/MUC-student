use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::Local;
use tauri::{Emitter, Manager};

use crate::application::dto::{
    AccountDto, AppSnapshotDto, LoginStateDto, PoolQuotaDto, PreferenceDto, RefreshStateDto,
};
use crate::application::error::{AppError, AppResult};
use crate::application::runtime::{AppRuntimeState, SharedRuntimeState};
use crate::application::services::account_traffic_service::AccountTrafficService;
use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::{CachedTrafficSnapshot, NetworkStatus, PortalAccount, UserPreferences};
use crate::domain::policies::account_selection::find_current_online_account;
use crate::domain::policies::traffic_math::{
    build_auto_switch_candidate, build_pool_quota_summary, build_status_card_order,
};
use crate::infrastructure::network::auth_portal_client::AuthPortalClient;
use crate::infrastructure::network::http_transport::HttpTransport;
use crate::infrastructure::network::network_status_service::NetworkStatusService;
use crate::infrastructure::network::self_service_panel_client::SelfServicePanelClient;
use crate::infrastructure::ocr::{
    ExternalWorkerOcrProvider, NativeRustOcrProvider, OcrProviderChain,
};
use crate::infrastructure::persistence::account_repository::{
    AccountRepository, AccountWithPassword,
};
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;
use crate::infrastructure::persistence::migration::MigrationService;
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
use crate::infrastructure::security::credential_vault::{CredentialVault, WindowsCredentialVault};
use crate::infrastructure::settings::AppSettings;
use crate::infrastructure::system::startup_service::StartupService;

#[derive(Clone)]
pub struct Backend {
    app: tauri::AppHandle,
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    app_state_repo: AppStateRepository,
    auth_client: AuthPortalClient,
    panel_client: SelfServicePanelClient,
    traffic_service: AccountTrafficService,
    network_status_service: Arc<NetworkStatusService>,
    startup_service: StartupService,
}

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInput {
    pub remark_name: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateInput {
    pub account_id: String,
    pub remark_name: String,
    pub username: String,
    pub password: Option<String>,
}

#[derive(Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferenceInput {
    pub minimize_to_tray_on_close: bool,
    pub launch_on_startup: bool,
    pub auto_switch_account_on_traffic_exhausted: bool,
}

impl Backend {
    const QUOTA_REFRESH_COOLDOWN_MINUTES: i64 = 30;

    pub fn build(app: tauri::AppHandle) -> AppResult<Self> {
        let settings = AppSettings::default();
        let resource_base_dir = app
            .path()
            .resource_dir()
            .map_err(|err| AppError::Storage(format!("无法定位资源目录：{err}")))?;
        let legacy_root = std::env::current_dir()?;
        let paths = RuntimePaths::new(
            app.path()
                .app_local_data_dir()
                .map_err(|err| AppError::Storage(format!("无法定位应用数据目录：{err}")))?,
            resource_base_dir,
            legacy_root,
        )?;
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

        let native_ocr = Arc::new(NativeRustOcrProvider::new(
            paths.ocr_detection_model_path(),
            paths.ocr_recognition_model_path(),
        ));
        let worker_ocr = Arc::new(ExternalWorkerOcrProvider::new(paths.ocr_worker_path()));
        let ocr_chain = Arc::new(OcrProviderChain::new(native_ocr, worker_ocr));
        let auth_transport = HttpTransport::new(settings.clone());
        let panel_transport = HttpTransport::new(settings.clone());
        let auth_client =
            AuthPortalClient::new(settings.clone(), auth_transport, ocr_chain.clone());
        let panel_client =
            SelfServicePanelClient::new(settings.clone(), panel_transport, ocr_chain);
        let traffic_service = AccountTrafficService::new(panel_client.clone());
        let network_status_service = Arc::new(NetworkStatusService::new(settings));
        let startup_service = StartupService::new(app.clone());
        let snapshots = restore_cached_snapshots(&account_store.cached_traffic_snapshots);
        let runtime = SharedRuntimeState::new(AppRuntimeState {
            account_store: account_store.clone(),
            app_state: app_state.clone(),
            preferences: preferences.clone(),
            network: NetworkStatus::default(),
            snapshots,
            selected_account_id: account_store.selected_account_id.clone(),
            current_online_account_id: account_store.current_online_account_id.clone(),
            login_running: false,
            refresh_running: false,
            logout_running: false,
            migration_version: 1,
        });
        let backend = Self {
            app,
            state: runtime,
            account_repo,
            app_state_repo,
            auth_client,
            panel_client,
            traffic_service,
            network_status_service,
            startup_service,
        };
        Ok(backend)
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

    pub async fn create_account(&self, input: AccountInput) -> AppResult<AppSnapshotDto> {
        let remark_name = require_input_text(&input.remark_name, "备注名")?;
        let username = require_input_text(&input.username, "账号")?;
        let password = require_input_text(&input.password, "密码")?;
        self.validate_panel_credentials(AccountWithPassword {
            account: PortalAccount {
                id: "__new_account_validation__".to_string(),
                remark_name: remark_name.clone(),
                username: username.clone(),
            },
            password: password.clone(),
        })
        .await?;
        let account = self
            .account_repo
            .add_account(&remark_name, &username, &password)?;
        self.refresh_runtime_from_disk()?;
        let _ = account;
        self.emit_state()
    }

    pub async fn update_account(&self, input: AccountUpdateInput) -> AppResult<AppSnapshotDto> {
        let remark_name = require_input_text(&input.remark_name, "备注名")?;
        let username = require_input_text(&input.username, "账号")?;
        let store = self.account_repo.load_store()?;
        let existing = self
            .account_repo
            .get_account_by_id(&store, &input.account_id)
            .ok_or_else(|| AppError::NotFound("找不到要编辑的账号".to_string()))?;
        let new_password = input
            .password
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string);
        let password = if let Some(password) = &new_password {
            password.clone()
        } else {
            self.account_repo
                .load_account_with_password(&existing)?
                .password
        };
        self.validate_panel_credentials(AccountWithPassword {
            account: PortalAccount {
                id: input.account_id.clone(),
                remark_name: remark_name.clone(),
                username: username.clone(),
            },
            password,
        })
        .await?;
        let account = self.account_repo.update_account(
            &input.account_id,
            &remark_name,
            &username,
            new_password.as_deref(),
        )?;
        self.refresh_runtime_from_disk()?;
        let _ = account;
        self.emit_state()
    }

    pub async fn delete_account(&self, account_id: String) -> AppResult<AppSnapshotDto> {
        let account = self.account_repo.delete_account(&account_id)?;
        self.refresh_runtime_from_disk()?;
        {
            let mut state = self.state.write();
            state.snapshots.remove(&account_id);
            if state.current_online_account_id == account_id {
                state.current_online_account_id.clear();
            }
        }
        let _ = account;
        self.emit_state()
    }

    pub async fn update_preferences(&self, input: PreferenceInput) -> AppResult<AppSnapshotDto> {
        let preferences = UserPreferences {
            minimize_to_tray_on_close: input.minimize_to_tray_on_close,
            launch_on_startup: input.launch_on_startup,
            auto_switch_account_on_traffic_exhausted: input
                .auto_switch_account_on_traffic_exhausted,
        };
        self.startup_service
            .set_launch_on_startup(preferences.launch_on_startup)?;
        self.app_state_repo.save_preferences(&preferences)?;
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
        let result = self.login_selected_account_inner().await;
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
        let result = self.logout_local_device_inner().await;
        {
            let mut state = self.state.write();
            state.logout_running = false;
        }
        self.emit_task_finished("logout")?;
        result?;
        self.emit_state()
    }

    async fn login_selected_account_inner(&self) -> AppResult<()> {
        let store = self.account_repo.load_store()?;
        let selected = self
            .account_repo
            .get_selected_account(&store)
            .ok_or_else(|| AppError::Validation("当前没有可用账号，请先添加账号".to_string()))?;
        let target = self.account_repo.load_account_with_password(&selected)?;

        let network = self.network_status_service.detect_network_status();
        {
            let mut state = self.state.write();
            state.network = network.clone();
        }
        let local_ip = if network.ip == "unknown" {
            None
        } else {
            Some(network.ip.as_str())
        };

        let all_accounts = self
            .account_repo
            .load_accounts_with_passwords(&store.accounts)?;
        if let Some(local_ip) = local_ip {
            let snapshots = self
                .traffic_service
                .fetch_balances(&all_accounts, Some(local_ip))
                .await;
            let current_online = find_current_online_account(&store.accounts, &snapshots);
            if let Some(current) = current_online {
                if current.id != target.account.id {
                    let current_with_password =
                        self.account_repo.load_account_with_password(&current)?;
                    self.panel_client
                        .logout_local_device(&current_with_password, local_ip)
                        .await?;
                }
            }
        }

        let mut login_result = self.auth_client.verify_login(&target).await?;
        if login_result.already_online {
            let current_id = {
                let state = self.state.read();
                state.current_online_account_id.clone()
            };
            if current_id == target.account.id {
                login_result.success = true;
                login_result.message = format!(
                    "当前 IP 已在线（{}），无需重复登录",
                    target.account.display_name()
                );
            } else {
                login_result.success = false;
                login_result.message =
                    "当前 IP 已在线，但无法确认是不是目标账号，请先本机下线后再登录".to_string();
            }
        }

        let mut app_state = self.app_state_repo.load_state()?;
        app_state.last_login_time = Some(login_result.checked_at);
        app_state.last_login_result = if login_result.success {
            "成功".to_string()
        } else {
            "失败".to_string()
        };
        app_state.last_login_message = login_result.message.clone();
        self.app_state_repo.save_state(&app_state)?;
        if login_result.success {
            self.app_state_repo.mark_account_used(&target.account.id)?;
        }
        self.refresh_runtime_from_disk()?;
        Ok(())
    }

    async fn logout_local_device_inner(&self) -> AppResult<()> {
        let network = self.network_status_service.detect_network_status();
        if network.ip == "unknown" || network.ip.trim().is_empty() {
            return Err(AppError::Validation(
                "本机内网 IP 未识别到，无法执行本机下线".to_string(),
            ));
        }
        let current_id = { self.state.read().current_online_account_id.clone() };
        if current_id.is_empty() {
            return Err(AppError::Validation("当前没有可下线的在线账号".to_string()));
        }
        let store = self.account_repo.load_store()?;
        let account = self
            .account_repo
            .get_account_by_id(&store, &current_id)
            .ok_or_else(|| AppError::NotFound("找不到当前在线账号".to_string()))?;
        let account = self.account_repo.load_account_with_password(&account)?;
        self
            .panel_client
            .logout_local_device(&account, &network.ip)
            .await?;
        self.run_refresh(true).await?;
        Ok(())
    }

    async fn run_refresh(&self, force: bool) -> AppResult<AppSnapshotDto> {
        if !force && self.is_quota_refresh_in_cooldown() {
            return self.emit_state();
        }
        {
            let mut state = self.state.write();
            if state.refresh_running {
                return Ok(self.build_snapshot()?);
            }
            state.refresh_running = true;
        }
        self.emit_task_started("refresh")?;
        let result = self.refresh_inner(force).await;
        {
            let mut state = self.state.write();
            state.refresh_running = false;
        }
        self.emit_task_finished("refresh")?;
        result?;
        self.emit_state()
    }

    async fn refresh_inner(&self, _force: bool) -> AppResult<()> {
        let network = self.network_status_service.detect_network_status();
        let local_ip = if network.ip == "unknown" {
            None
        } else {
            Some(network.ip.as_str())
        };
        let store = self.account_repo.load_store()?;
        let accounts = self
            .account_repo
            .load_accounts_with_passwords(&store.accounts)?;
        {
            let mut state = self.state.write();
            state.network = network.clone();
            state.snapshots.clear();
            let now = Local::now();
            for account in &store.accounts {
                state.snapshots.insert(
                    account.id.clone(),
                    AccountTrafficSnapshot::loading(account.id.clone(), now),
                );
            }
        }
        self.emit_state()?;
        let snapshots = self
            .traffic_service
            .fetch_balances(&accounts, local_ip)
            .await;
        let snapshot_map = AccountTrafficService::to_snapshot_map(snapshots);
        let mut current_online_id = String::new();
        for account in &store.accounts {
            if snapshot_map
                .get(&account.id)
                .and_then(|snapshot| snapshot.matched_local_ip_device.as_ref())
                .is_some()
            {
                current_online_id = account.id.clone();
                break;
            }
        }
        let order = build_status_card_order(
            &store,
            &snapshot_map,
            &current_online_id,
            &store.status_card_order_snapshot,
        );
        let cached = to_cached_snapshots(&snapshot_map);
        self.account_repo.save_cached_traffic_snapshots(
            cached,
            current_online_id.clone(),
            order,
        )?;
        let mut app_state = self.app_state_repo.load_state()?;
        app_state.last_quota_refresh_time = Some(Local::now());
        self.app_state_repo.save_state(&app_state)?;
        self.refresh_runtime_from_disk()?;
        {
            let mut state = self.state.write();
            state.snapshots = snapshot_map;
            state.current_online_account_id = current_online_id.clone();
            state.account_store.current_online_account_id = current_online_id;
        }
        self.try_auto_switch().await?;
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
        state.selected_account_id = account_store.selected_account_id.clone();
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
        let (used, total, included, progress) =
            build_pool_quota_summary(&state.account_store, &state.snapshots);
        let accounts = state
            .account_store
            .accounts
            .iter()
            .map(|account| {
                AccountDto::from_store(
                    account,
                    &state.account_store,
                    state.snapshots.get(&account.id),
                )
            })
            .collect();
        let mut login_state = LoginStateDto::from(&state.app_state);
        login_state.running = state.login_running;
        let mut refresh_state = RefreshStateDto::from(&state.app_state);
        refresh_state.running = state.refresh_running || state.logout_running;
        Ok(AppSnapshotDto {
            network: state.network.clone(),
            accounts,
            selected_account_id: state.account_store.selected_account_id.clone(),
            current_online_account_id: state.current_online_account_id.clone(),
            pool_quota: PoolQuotaDto {
                used_traffic_text: used,
                product_balance_text: total,
                included_package_text: included,
                progress_percent: progress,
            },
            login_state,
            refresh_state,
            preferences: PreferenceDto::from(&state.preferences),
        })
    }

    fn emit_state(&self) -> AppResult<AppSnapshotDto> {
        let snapshot = self.build_snapshot()?;
        self.app
            .emit("app://state-updated", &snapshot)
            .map_err(|err| AppError::System(format!("发送状态事件失败：{err}")))?;
        Ok(snapshot)
    }

    fn emit_task_started(&self, task: &str) -> AppResult<()> {
        self.app
            .emit("app://task-started", task)
            .map_err(|err| AppError::System(format!("发送任务开始事件失败：{err}")))
    }

    fn emit_task_finished(&self, task: &str) -> AppResult<()> {
        self.app
            .emit("app://task-finished", task)
            .map_err(|err| AppError::System(format!("发送任务结束事件失败：{err}")))
    }

    async fn validate_panel_credentials(&self, account: AccountWithPassword) -> AppResult<()> {
        self.panel_client
            .fetch_authenticated_html(&account, "/home")
            .await?;
        Ok(())
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

fn restore_cached_snapshots(
    cached: &BTreeMap<String, CachedTrafficSnapshot>,
) -> BTreeMap<String, AccountTrafficSnapshot> {
    cached
        .iter()
        .map(|(account_id, snapshot)| {
            (
                account_id.clone(),
                AccountTrafficSnapshot {
                    account_id: account_id.clone(),
                    used_traffic_text: snapshot.used_traffic_text.clone(),
                    product_balance_text: snapshot.product_balance_text.clone(),
                    included_package_text: snapshot.included_package_text.clone(),
                    online_device_count_text: snapshot.online_device_count_text.clone(),
                    package_text: snapshot.package_text.clone(),
                    status_text: snapshot.status_text.clone(),
                    detail_text: snapshot.detail_text.clone(),
                    queried_at: snapshot.queried_at.unwrap_or_else(Local::now),
                    online_devices: Vec::new(),
                    matched_local_ip_device: None,
                    progress_percent: snapshot.progress_percent,
                },
            )
        })
        .collect()
}

fn to_cached_snapshots(
    snapshots: &BTreeMap<String, AccountTrafficSnapshot>,
) -> BTreeMap<String, CachedTrafficSnapshot> {
    snapshots
        .iter()
        .filter(|(_, snapshot)| {
            snapshot.status_text != "查询中..." && snapshot.status_text != "查询失败"
        })
        .map(|(account_id, snapshot)| {
            (
                account_id.clone(),
                CachedTrafficSnapshot {
                    used_traffic_text: snapshot.used_traffic_text.clone(),
                    product_balance_text: snapshot.product_balance_text.clone(),
                    included_package_text: snapshot.included_package_text.clone(),
                    online_device_count_text: snapshot.online_device_count_text.clone(),
                    package_text: snapshot.package_text.clone(),
                    status_text: snapshot.status_text.clone(),
                    detail_text: snapshot.detail_text.clone(),
                    queried_at: Some(snapshot.queried_at),
                    progress_percent: snapshot.progress_percent,
                },
            )
        })
        .collect()
}

fn require_input_text(value: &str, field_name: &str) -> AppResult<String> {
    let clean = value.trim();
    if clean.is_empty() {
        Err(AppError::Validation(format!("{field_name}不能为空")))
    } else {
        Ok(clean.to_string())
    }
}
