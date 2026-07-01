use std::sync::Arc;

use chrono::Local;

use crate::application::error::{AppError, AppResult};
use crate::application::runtime::SharedRuntimeState;
use crate::application::runtime_refresh::refresh_runtime_from_disk;
use crate::application::services::portal_snapshot_service::username_matches;
use crate::domain::models::{CachedTrafficSnapshot, LoginResult, PortalAccount};
use crate::infrastructure::network::legacy_portal_auth_client::{
    is_portal_arrearage_response, LegacyPortalAuthClient,
};
use crate::infrastructure::network::legacy_portal_status_client::LegacyPortalStatusClient;
use crate::infrastructure::network::network_status_service::NetworkStatusDetector;
use crate::infrastructure::persistence::account_repository::{
    AccountRepository, AccountWithPassword,
};
use crate::infrastructure::persistence::account_snapshot_repository::AccountSnapshotRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;

#[derive(Clone)]
pub struct SessionService {
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    snapshot_repo: AccountSnapshotRepository,
    app_state_repo: AppStateRepository,
    auth_client: LegacyPortalAuthClient,
    portal_status_client: LegacyPortalStatusClient,
    network_status_service: Arc<dyn NetworkStatusDetector>,
}

enum LocalOnlineAccount {
    Known(PortalAccount),
    Unknown,
}

impl LocalOnlineAccount {
    fn known_account_id(&self) -> Option<String> {
        match self {
            Self::Known(account) => Some(account.id.clone()),
            Self::Unknown => None,
        }
    }
}

impl SessionService {
    pub fn new(
        state: SharedRuntimeState,
        account_repo: AccountRepository,
        snapshot_repo: AccountSnapshotRepository,
        app_state_repo: AppStateRepository,
        auth_client: LegacyPortalAuthClient,
        portal_status_client: LegacyPortalStatusClient,
        network_status_service: Arc<dyn NetworkStatusDetector>,
    ) -> Self {
        Self {
            state,
            account_repo,
            snapshot_repo,
            app_state_repo,
            auth_client,
            portal_status_client,
            network_status_service,
        }
    }

    pub async fn login_selected_account_inner(&self) -> AppResult<()> {
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

        let mut current_online_account = None;
        if let Some(local_ip) = local_ip {
            let current_online = self
                .detect_local_online_account(&store.accounts, local_ip)
                .await;
            if let Some(current) = current_online {
                if let LocalOnlineAccount::Known(account) = &current {
                    {
                        let mut state = self.state.write();
                        state.current_online_account_id = account.id.clone();
                        state.account_store.current_online_account_id = account.id.clone();
                    }
                } else {
                    let mut state = self.state.write();
                    state.current_online_account_id.clear();
                    state.account_store.current_online_account_id.clear();
                }
                current_online_account = Some(current);
            }
        }

        let detected_current_id = current_online_account
            .as_ref()
            .and_then(|account| account.known_account_id())
            .unwrap_or_default();
        let mut login_result = match current_online_account {
            Some(LocalOnlineAccount::Known(current)) if current.id == target.account.id => {
                LoginResult {
                    success: true,
                    message: format!(
                        "当前 IP 已在线（{}），无需重复登录",
                        target.account.display_name()
                    ),
                    login_url: String::new(),
                    hidden_fields: Default::default(),
                    response_text: String::new(),
                    checked_at: Local::now(),
                    already_online: false,
                }
            }
            Some(LocalOnlineAccount::Known(_)) | Some(LocalOnlineAccount::Unknown) => {
                self.auth_client.login_target_account(&target).await?
            }
            None => self.auth_client.verify_login(&target).await?,
        };
        if login_result.already_online {
            if detected_current_id == target.account.id
                || self.confirm_target_online(&target, local_ip).await
            {
                login_result.success = true;
                login_result.message = format!(
                    "当前 IP 已在线（{}），无需重复登录",
                    target.account.display_name()
                );
            } else {
                login_result = self
                    .switch_already_online_to_target(&store.accounts, &target, local_ip)
                    .await?;
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
        if !login_result.success && is_portal_arrearage_response(&login_result.response_text) {
            self.persist_arrearage_snapshot(&target, login_result.checked_at)?;
        }
        if login_result.success {
            self.confirm_and_persist_online_account(&target, local_ip)
                .await?;
            self.app_state_repo.mark_account_used(&target.account.id)?;
        }
        self.refresh_runtime_from_disk()?;
        Ok(())
    }

    async fn switch_already_online_to_target(
        &self,
        accounts: &[PortalAccount],
        target: &AccountWithPassword,
        local_ip: Option<&str>,
    ) -> AppResult<LoginResult> {
        let Some(local_ip) = local_ip else {
            return self.auth_client.login_target_account(target).await;
        };
        let Some(current) = self.detect_local_online_account(accounts, local_ip).await else {
            return self.auth_client.login_target_account(target).await;
        };
        if matches!(&current, LocalOnlineAccount::Known(account) if account.id == target.account.id)
        {
            return Ok(LoginResult {
                success: true,
                message: format!(
                    "当前 IP 已在线（{}），无需重复登录",
                    target.account.display_name()
                ),
                login_url: String::new(),
                hidden_fields: Default::default(),
                response_text: String::new(),
                checked_at: Local::now(),
                already_online: false,
            });
        }
        self.auth_client.login_target_account(target).await
    }

    fn persist_arrearage_snapshot(
        &self,
        target: &AccountWithPassword,
        checked_at: chrono::DateTime<Local>,
    ) -> AppResult<()> {
        let store = self.account_repo.load_store()?;
        let previous = store
            .cached_traffic_snapshots
            .get(&target.account.id)
            .cloned()
            .unwrap_or_default();
        let snapshot = CachedTrafficSnapshot {
            used_traffic_text: "70.00GB".to_string(),
            product_balance_text: "70.00GB".to_string(),
            included_package_text: String::new(),
            package_total_text: String::new(),
            package_available_text: String::new(),
            online_device_count_text: if previous.online_device_count_text.trim().is_empty() {
                "-".to_string()
            } else {
                previous.online_device_count_text
            },
            package_text: if previous.package_text.trim().is_empty() {
                "-".to_string()
            } else {
                previous.package_text
            },
            status_text: "已耗尽".to_string(),
            detail_text: "Portal 返回欠费，按流量 100% 耗尽处理".to_string(),
            queried_at: Some(checked_at),
            progress_percent: Some(100.0),
        };
        self.snapshot_repo
            .save_cached_snapshot(&store.accounts, &target.account.id, snapshot)
    }

    pub async fn logout_local_device_inner(&self) -> AppResult<()> {
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
        self.auth_client.logout_current_ip(&account).await?;
        Ok(())
    }

    async fn detect_local_online_account(
        &self,
        accounts: &[PortalAccount],
        local_ip: &str,
    ) -> Option<LocalOnlineAccount> {
        let success_info = self.portal_status_client.fetch_success_info().await.ok()?;
        if success_info.ip.trim() != local_ip.trim() {
            return None;
        }

        let account = accounts
            .iter()
            .find(|account| username_matches(&account.username, &success_info.username))
            .cloned();
        Some(match account {
            Some(account) => LocalOnlineAccount::Known(account),
            None => LocalOnlineAccount::Unknown,
        })
    }

    async fn confirm_target_online(
        &self,
        target: &AccountWithPassword,
        local_ip: Option<&str>,
    ) -> bool {
        let Ok(info) = self.portal_status_client.fetch_success_info().await else {
            return false;
        };
        if let Some(local_ip) = local_ip {
            if info.ip.trim() != local_ip.trim() {
                return false;
            }
        }
        username_matches(&target.account.username, &info.username)
    }

    async fn confirm_and_persist_online_account(
        &self,
        target: &AccountWithPassword,
        local_ip: Option<&str>,
    ) -> AppResult<()> {
        if !self.confirm_target_online(target, local_ip).await {
            return Err(AppError::Network(format!(
                "Portal 登录返回成功，但成功页没有确认目标账号在线：{}",
                target.account.display_name()
            )));
        }
        let store = self.account_repo.load_store()?;
        self.snapshot_repo
            .set_current_online_account_id(&store.accounts, target.account.id.clone())?;
        {
            let mut state = self.state.write();
            state.current_online_account_id = target.account.id.clone();
            state.account_store.current_online_account_id = target.account.id.clone();
        }
        Ok(())
    }

    fn refresh_runtime_from_disk(&self) -> AppResult<()> {
        refresh_runtime_from_disk(&self.state, &self.account_repo, &self.app_state_repo)
    }
}
