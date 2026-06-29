use std::collections::HashSet;
use std::sync::Arc;

use chrono::Local;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::application::dto::AppSnapshotDto;
use crate::application::error::{AppError, AppResult};
use crate::application::platform::AppEventSink;
use crate::application::runtime::SharedRuntimeState;
use crate::application::runtime_refresh::refresh_runtime_from_disk;
use crate::application::services::account_traffic_service::AccountTrafficService;
use crate::application::services::portal_snapshot_service::{
    build_single_success_snapshot_with_online_info, username_matches,
};
use crate::application::services::snapshot_mapper::{
    build_app_snapshot, restore_cached_snapshots, to_cached_snapshots,
};
use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::AccountStore;
use crate::domain::policies::traffic_math::build_status_card_order;
use crate::infrastructure::network::legacy_portal_status_client::LegacyPortalStatusClient;
use crate::infrastructure::network::network_status_service::NetworkStatusDetector;
use crate::infrastructure::network::self_service_panel_client::SelfServicePanelClient;
use crate::infrastructure::parsers::legacy_portal_online_info_parser::LegacyPortalOnlineInfo;
use crate::infrastructure::persistence::account_repository::AccountRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;

#[derive(Clone)]
pub struct DashboardRefreshService {
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    app_state_repo: AppStateRepository,
    portal_status_client: LegacyPortalStatusClient,
    panel_client: SelfServicePanelClient,
    network_status_service: Arc<dyn NetworkStatusDetector>,
    event_sink: Arc<dyn AppEventSink>,
}

pub struct DashboardRefreshDependencies {
    pub state: SharedRuntimeState,
    pub account_repo: AccountRepository,
    pub app_state_repo: AppStateRepository,
    pub portal_status_client: LegacyPortalStatusClient,
    pub panel_client: SelfServicePanelClient,
    pub network_status_service: Arc<dyn NetworkStatusDetector>,
    pub event_sink: Arc<dyn AppEventSink>,
}

impl DashboardRefreshService {
    pub fn new(deps: DashboardRefreshDependencies) -> Self {
        Self {
            state: deps.state,
            account_repo: deps.account_repo,
            app_state_repo: deps.app_state_repo,
            portal_status_client: deps.portal_status_client,
            panel_client: deps.panel_client,
            network_status_service: deps.network_status_service,
            event_sink: deps.event_sink,
        }
    }

    pub async fn refresh_accounts(&self) -> AppResult<()> {
        let network = self.network_status_service.detect_network_status();
        let local_ip = if network.ip == "unknown" {
            None
        } else {
            Some(network.ip.as_str())
        };
        let store = self.account_repo.load_store()?;
        let mut snapshot_map = restore_cached_snapshots(&store.cached_traffic_snapshots);
        {
            let mut state = self.state.write();
            state.network = network.clone();
            state.snapshots = snapshot_map.clone();
        }
        self.emit_state()?;

        let online_probe = self.check_local_online(local_ip).await;
        let mut current_online_id = String::new();
        let mut refreshed_account_ids = HashSet::new();
        if !online_probe.is_offline() {
            for (account_id, snapshot) in self.refresh_cached_panel_sessions(&store, local_ip).await
            {
                if current_online_id.is_empty() && snapshot.matched_local_ip_device.is_some() {
                    current_online_id = account_id.clone();
                }
                refreshed_account_ids.insert(account_id.clone());
                snapshot_map.insert(account_id, snapshot);
            }
        }

        if current_online_id.is_empty() && online_probe.is_online() {
            if let Ok(info) = self.portal_status_client.fetch_success_info().await {
                let success_account =
                    local_ip
                        .filter(|ip| info.ip.trim() == ip.trim())
                        .and_then(|_| {
                            store
                                .accounts
                                .iter()
                                .find(|account| username_matches(&account.username, &info.username))
                        });
                if let Some(account) = success_account {
                    current_online_id = account.id.clone();
                    let snapshot = match self
                        .panel_client
                        .fetch_sso_html(&account.id, &info.username, "/home")
                        .await
                        .and_then(|html| {
                            AccountTrafficService::snapshot_from_panel_home(
                                account, &html, local_ip,
                            )
                        }) {
                        Ok(snapshot) => snapshot,
                        Err(_) => build_single_success_snapshot_with_online_info(
                            account,
                            &info,
                            online_probe.online_info(),
                            store.cached_traffic_snapshots.get(&account.id),
                        ),
                    };
                    refreshed_account_ids.insert(account.id.clone());
                    snapshot_map.insert(account.id.clone(), snapshot);
                }
            }
        } else if current_online_id.is_empty() && online_probe.is_unknown() {
            current_online_id = store.current_online_account_id.clone();
        }
        if !online_probe.is_offline() {
            mark_unrefreshed_non_current_accounts_failed(
                &store,
                &mut snapshot_map,
                &refreshed_account_ids,
                &current_online_id,
            );
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
        Ok(())
    }

    async fn refresh_cached_panel_sessions(
        &self,
        store: &AccountStore,
        local_ip: Option<&str>,
    ) -> Vec<(String, AccountTrafficSnapshot)> {
        let semaphore = Arc::new(Semaphore::new(
            AccountTrafficService::DEFAULT_PANEL_QUERY_CONCURRENCY,
        ));
        let mut join_set = JoinSet::new();
        let local_ip = local_ip.map(str::to_string);
        for account in store.accounts.iter().cloned() {
            let panel_client = self.panel_client.clone();
            let local_ip = local_ip.clone();
            let sem = semaphore.clone();
            join_set.spawn(async move {
                let _permit = sem.acquire_owned().await.ok()?;
                let html = match panel_client
                    .fetch_cached_session_html(&account.id, "/home")
                    .await
                {
                    Ok(Some(html)) => html,
                    _ => return None,
                };
                AccountTrafficService::snapshot_from_panel_home(
                    &account,
                    &html,
                    local_ip.as_deref(),
                )
                .ok()
                .map(|snapshot| (account.id.clone(), snapshot))
            });
        }

        let mut snapshots = Vec::new();
        while let Some(result) = join_set.join_next().await {
            if let Ok(Some(snapshot)) = result {
                snapshots.push(snapshot);
            }
        }
        snapshots
    }

    async fn check_local_online(&self, local_ip: Option<&str>) -> LocalOnlineProbe {
        let Some(local_ip) = local_ip else {
            return LocalOnlineProbe::Unknown;
        };
        match self.portal_status_client.fetch_online_info().await {
            Ok(info) if info.ip.trim() == local_ip.trim() => LocalOnlineProbe::Online(info),
            Ok(_) | Err(AppError::NotFound(_)) => LocalOnlineProbe::Offline,
            Err(_) => LocalOnlineProbe::Unknown,
        }
    }

    fn refresh_runtime_from_disk(&self) -> AppResult<()> {
        refresh_runtime_from_disk(&self.state, &self.account_repo, &self.app_state_repo)
    }

    fn emit_state(&self) -> AppResult<AppSnapshotDto> {
        let state = self.state.read();
        let snapshot = build_app_snapshot(&state);
        self.event_sink.state_updated(&snapshot)?;
        Ok(snapshot)
    }
}

fn mark_unrefreshed_non_current_accounts_failed(
    store: &AccountStore,
    snapshot_map: &mut std::collections::BTreeMap<String, AccountTrafficSnapshot>,
    refreshed_account_ids: &HashSet<String>,
    current_online_id: &str,
) {
    for account in &store.accounts {
        if account.id == current_online_id || refreshed_account_ids.contains(&account.id) {
            continue;
        }
        if let Some(snapshot) = snapshot_map.get_mut(&account.id) {
            snapshot.status_text = "同步失败".to_string();
            snapshot.detail_text = "本次没有拿到自助面板数据，显示上次同步结果".to_string();
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LocalOnlineProbe {
    Online(LegacyPortalOnlineInfo),
    Offline,
    Unknown,
}

impl LocalOnlineProbe {
    fn is_online(&self) -> bool {
        matches!(self, Self::Online(_))
    }

    fn is_offline(&self) -> bool {
        matches!(self, Self::Offline)
    }

    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    fn online_info(&self) -> Option<&LegacyPortalOnlineInfo> {
        match self {
            Self::Online(info) => Some(info),
            Self::Offline | Self::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use chrono::{Local, TimeZone};

    use super::mark_unrefreshed_non_current_accounts_failed;
    use crate::domain::models::traffic::AccountTrafficSnapshot;
    use crate::domain::models::{AccountStore, PortalAccount};

    #[test]
    fn marks_unrefreshed_non_current_cached_snapshot_failed_without_touching_time() {
        let old_time = Local.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
        let current_account = PortalAccount {
            id: "current".to_string(),
            remark_name: "当前".to_string(),
            username: "20260001".to_string(),
        };
        let stale_account = PortalAccount {
            id: "stale".to_string(),
            remark_name: "失效".to_string(),
            username: "20260002".to_string(),
        };
        let store = AccountStore {
            accounts: vec![current_account, stale_account.clone()],
            current_online_account_id: "current".to_string(),
            ..Default::default()
        };
        let mut snapshot_map = BTreeMap::new();
        snapshot_map.insert(
            stale_account.id.clone(),
            AccountTrafficSnapshot {
                account_id: stale_account.id.clone(),
                used_traffic_text: "10.00GB".to_string(),
                product_balance_text: "70.00GB".to_string(),
                included_package_text: "70GB".to_string(),
                package_total_text: String::new(),
                package_available_text: String::new(),
                online_device_count_text: "1".to_string(),
                package_text: "校园网".to_string(),
                status_text: "已同步".to_string(),
                detail_text: "计费策略：免费70GB".to_string(),
                queried_at: old_time,
                online_devices: Vec::new(),
                matched_local_ip_device: None,
                progress_percent: Some(14.3),
            },
        );

        mark_unrefreshed_non_current_accounts_failed(
            &store,
            &mut snapshot_map,
            &HashSet::from(["current".to_string()]),
            "current",
        );

        let snapshot = snapshot_map.get("stale").expect("stale snapshot");
        assert_eq!(snapshot.status_text, "同步失败");
        assert_eq!(snapshot.queried_at, old_time);
        assert_eq!(snapshot.used_traffic_text, "10.00GB");
    }
}
