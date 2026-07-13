use std::collections::HashSet;
use std::sync::Arc;

use chrono::Local;

use crate::application::dto::AppSnapshotDto;
use crate::application::error::{AppError, AppResult};
use crate::application::platform::AppEventSink;
use crate::application::runtime::SharedRuntimeState;
use crate::application::runtime_refresh::refresh_runtime_from_disk;
use crate::application::services::account_traffic_service::snapshot_from_panel_home;
use crate::application::services::portal_snapshot_service::{
    apply_success_page_unlimited_plan, build_single_success_snapshot_with_online_info,
    ipv4_matches, username_matches,
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
use crate::infrastructure::persistence::account_snapshot_repository::AccountSnapshotRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;

#[derive(Clone)]
pub struct DashboardRefreshService {
    state: SharedRuntimeState,
    account_repo: AccountRepository,
    snapshot_repo: AccountSnapshotRepository,
    app_state_repo: AppStateRepository,
    portal_status_client: LegacyPortalStatusClient,
    panel_client: SelfServicePanelClient,
    network_status_service: Arc<dyn NetworkStatusDetector>,
    event_sink: Arc<dyn AppEventSink>,
}

impl DashboardRefreshService {
    pub fn new(
        state: SharedRuntimeState,
        account_repo: AccountRepository,
        snapshot_repo: AccountSnapshotRepository,
        app_state_repo: AppStateRepository,
        portal_status_client: LegacyPortalStatusClient,
        panel_client: SelfServicePanelClient,
        network_status_service: Arc<dyn NetworkStatusDetector>,
        event_sink: Arc<dyn AppEventSink>,
    ) -> Self {
        Self {
            state,
            account_repo,
            snapshot_repo,
            app_state_repo,
            portal_status_client,
            panel_client,
            network_status_service,
            event_sink,
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

        if online_probe.is_online() {
            if let Ok(info) = self.portal_status_client.fetch_success_info().await {
                if local_ip.is_some_and(|local_ip| ipv4_matches(local_ip, &info.ip)) {
                    let success_account = store
                        .accounts
                        .iter()
                        .find(|account| username_matches(&account.username, &info.username));
                    if let Some(account) = success_account {
                        current_online_id = account.id.clone();
                        let mut snapshot = match self
                            .panel_client
                            .fetch_sso_html(&account.id, &info.username, "/home")
                            .await
                            .and_then(|html| {
                                snapshot_from_panel_home(account, &html, Some(info.ip.trim()))
                            }) {
                            Ok(snapshot) => snapshot,
                            Err(_) => build_single_success_snapshot_with_online_info(
                                account,
                                &info,
                                online_probe.online_info(),
                                store.cached_traffic_snapshots.get(&account.id),
                            ),
                        };
                        apply_success_page_unlimited_plan(&mut snapshot, &info.billing_policy);
                        refreshed_account_ids.insert(account.id.clone());
                        snapshot_map.insert(account.id.clone(), snapshot);
                    }
                }
            }
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
        self.snapshot_repo.save_cached_traffic_snapshots(
            &store.accounts,
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

    async fn check_local_online(&self, local_ip: Option<&str>) -> LocalOnlineProbe {
        let Some(local_ip) = local_ip else {
            return LocalOnlineProbe::Unknown;
        };
        match self.portal_status_client.fetch_online_info().await {
            Ok(info) if info.ip.trim().is_empty() => LocalOnlineProbe::Offline,
            Ok(info) if ipv4_matches(local_ip, &info.ip) => LocalOnlineProbe::Online(info),
            Ok(_) => LocalOnlineProbe::Offline,
            Err(AppError::NotFound(_)) => LocalOnlineProbe::Offline,
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
                is_unlimited_plan: false,
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
