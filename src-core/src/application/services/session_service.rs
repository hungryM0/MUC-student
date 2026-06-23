use std::sync::Arc;

use chrono::Local;

use crate::application::error::{AppError, AppResult};
use crate::application::runtime::SharedRuntimeState;
use crate::application::services::portal_snapshot_service::username_matches;
use crate::application::services::snapshot_mapper::restore_cached_snapshots;
use crate::domain::models::{LoginResult, PortalAccount};
use crate::infrastructure::network::legacy_portal_auth_client::LegacyPortalAuthClient;
use crate::infrastructure::network::legacy_portal_status_client::LegacyPortalStatusClient;
use crate::infrastructure::network::network_status_service::NetworkStatusService;
use crate::infrastructure::persistence::account_repository::{
    AccountRepository, AccountWithPassword,
};
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;

#[derive(Clone)]
pub struct SessionService {
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    app_state_repo: AppStateRepository,
    auth_client: LegacyPortalAuthClient,
    portal_status_client: LegacyPortalStatusClient,
    network_status_service: Arc<NetworkStatusService>,
}

impl SessionService {
    pub fn new(
        state: SharedRuntimeState,
        account_repo: AccountRepository,
        app_state_repo: AppStateRepository,
        auth_client: LegacyPortalAuthClient,
        portal_status_client: LegacyPortalStatusClient,
        network_status_service: Arc<NetworkStatusService>,
    ) -> Self {
        Self {
            state,
            account_repo,
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
                .detect_current_online_account_fast(&store.accounts, local_ip)
                .await;
            if let Some(current) = current_online {
                {
                    let mut state = self.state.write();
                    state.current_online_account_id = current.id.clone();
                    state.account_store.current_online_account_id = current.id.clone();
                }
                current_online_account = Some(current);
            }
        }

        let detected_current_id = current_online_account
            .as_ref()
            .map(|account| account.id.clone())
            .unwrap_or_default();
        let mut login_result = if let Some(current) = current_online_account {
            if current.id == target.account.id {
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
            } else {
                self.auth_client.login_target_account(&target).await?
            }
        } else {
            self.auth_client.verify_login(&target).await?
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
            self.confirm_and_persist_online_account(&target, local_ip)
                .await?;
            self.app_state_repo.mark_account_used(&target.account.id)?;
        }
        self.refresh_runtime_from_disk()?;
        Ok(())
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

    async fn detect_current_online_account_fast(
        &self,
        accounts: &[PortalAccount],
        local_ip: &str,
    ) -> Option<PortalAccount> {
        let online_info = self.portal_status_client.fetch_online_info().await.ok()?;
        if online_info.ip.trim() != local_ip.trim() {
            return None;
        }
        accounts
            .iter()
            .find(|account| username_matches(&account.username, &online_info.username))
            .cloned()
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
        let mut store = self.account_repo.load_store()?;
        store.current_online_account_id = target.account.id.clone();
        self.account_repo.save_store(&store)?;
        {
            let mut state = self.state.write();
            state.current_online_account_id = target.account.id.clone();
            state.account_store.current_online_account_id = target.account.id.clone();
        }
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
}
