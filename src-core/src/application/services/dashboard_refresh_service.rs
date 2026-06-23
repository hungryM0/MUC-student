use std::sync::Arc;

use chrono::Local;

use crate::application::dto::AppSnapshotDto;
use crate::application::error::AppResult;
use crate::application::platform::AppEventSink;
use crate::application::runtime::SharedRuntimeState;
use crate::application::services::account_traffic_service::AccountTrafficService;
use crate::application::services::portal_snapshot_service::{
    username_matches, PortalSnapshotService,
};
use crate::application::services::snapshot_mapper::{
    build_app_snapshot, restore_cached_snapshots, to_cached_snapshots,
};
use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::policies::traffic_math::build_status_card_order;
use crate::infrastructure::network::legacy_portal_status_client::LegacyPortalStatusClient;
use crate::infrastructure::network::network_status_service::NetworkStatusService;
use crate::infrastructure::persistence::account_repository::AccountRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;

#[derive(Clone)]
pub struct DashboardRefreshService {
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    app_state_repo: AppStateRepository,
    portal_status_client: LegacyPortalStatusClient,
    traffic_service: AccountTrafficService,
    portal_snapshot_service: PortalSnapshotService,
    network_status_service: Arc<NetworkStatusService>,
    event_sink: Arc<dyn AppEventSink>,
}

pub struct DashboardRefreshDependencies {
    pub state: SharedRuntimeState,
    pub account_repo: AccountRepository,
    pub app_state_repo: AppStateRepository,
    pub portal_status_client: LegacyPortalStatusClient,
    pub traffic_service: AccountTrafficService,
    pub portal_snapshot_service: PortalSnapshotService,
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
            traffic_service: deps.traffic_service,
            portal_snapshot_service: deps.portal_snapshot_service,
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

        let success_info = self.portal_status_client.fetch_success_info().await.ok();
        let success_account = success_info.as_ref().and_then(|info| {
            local_ip
                .filter(|ip| info.ip.trim() == ip.trim())
                .and_then(|_| {
                    store
                        .accounts
                        .iter()
                        .find(|account| username_matches(&account.username, &info.username))
                })
        });
        let current_account = success_account
            .and_then(|account| self.account_repo.load_account_with_password(account).ok());
        let mut current_online_id = success_account
            .map(|account| account.id.clone())
            .unwrap_or_default();
        let snapshot_map = if let Some(current_account) = current_account {
            self.portal_snapshot_service
                .fetch_balances_with_probe(
                    &store.accounts,
                    &store.cached_traffic_snapshots,
                    current_account,
                )
                .await?
        } else {
            let snapshots = self
                .traffic_service
                .fetch_balances(&accounts, local_ip)
                .await;
            AccountTrafficService::to_snapshot_map(snapshots)
        };
        for account in &store.accounts {
            if !current_online_id.is_empty() {
                break;
            }
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

    fn emit_state(&self) -> AppResult<AppSnapshotDto> {
        let state = self.state.read();
        let snapshot = build_app_snapshot(&state);
        self.event_sink.state_updated(&snapshot)?;
        Ok(snapshot)
    }
}
