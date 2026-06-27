use std::sync::Arc;

use chrono::Local;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::application::dto::AppSnapshotDto;
use crate::application::error::AppResult;
use crate::application::platform::AppEventSink;
use crate::application::runtime::SharedRuntimeState;
use crate::application::services::account_traffic_service::AccountTrafficService;
use crate::application::services::portal_snapshot_service::{
    build_single_success_snapshot, username_matches,
};
use crate::application::services::snapshot_mapper::{
    build_app_snapshot, restore_cached_snapshots, to_cached_snapshots,
};
use crate::domain::models::AccountStore;
use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::policies::traffic_math::build_status_card_order;
use crate::infrastructure::network::legacy_portal_status_client::LegacyPortalStatusClient;
use crate::infrastructure::network::network_status_service::NetworkStatusService;
use crate::infrastructure::network::self_service_panel_client::SelfServicePanelClient;
use crate::infrastructure::persistence::account_repository::AccountRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;

#[derive(Clone)]
pub struct DashboardRefreshService {
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    app_state_repo: AppStateRepository,
    portal_status_client: LegacyPortalStatusClient,
    panel_client: SelfServicePanelClient,
    network_status_service: Arc<NetworkStatusService>,
    event_sink: Arc<dyn AppEventSink>,
}

pub struct DashboardRefreshDependencies {
    pub state: SharedRuntimeState,
    pub account_repo: AccountRepository,
    pub app_state_repo: AppStateRepository,
    pub portal_status_client: LegacyPortalStatusClient,
    pub panel_client: SelfServicePanelClient,
    pub network_status_service: Arc<NetworkStatusService>,
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

        let mut current_online_id = String::new();
        for (account_id, snapshot) in self.refresh_cached_panel_sessions(&store, local_ip).await {
            if current_online_id.is_empty() && snapshot.matched_local_ip_device.is_some() {
                current_online_id = account_id.clone();
            }
            snapshot_map.insert(account_id, snapshot);
        }

        if current_online_id.is_empty() {
            let success_result = self.portal_status_client.fetch_success_info().await;
            match success_result {
                Ok(info) => {
                    let success_account = local_ip
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
                            Err(_) => build_single_success_snapshot(
                                account,
                                &info,
                                store.cached_traffic_snapshots.get(&account.id),
                            ),
                        };
                        snapshot_map.insert(account.id.clone(), snapshot);
                    }
                }
                Err(_) => {
                    current_online_id = store.current_online_account_id.clone();
                }
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

    fn emit_state(&self) -> AppResult<AppSnapshotDto> {
        let state = self.state.read();
        let snapshot = build_app_snapshot(&state);
        self.event_sink.state_updated(&snapshot)?;
        Ok(snapshot)
    }
}
