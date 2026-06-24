use std::sync::Arc;

use chrono::Local;

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
    network_status_service: Arc<NetworkStatusService>,
    event_sink: Arc<dyn AppEventSink>,
}

pub struct DashboardRefreshDependencies {
    pub state: SharedRuntimeState,
    pub account_repo: AccountRepository,
    pub app_state_repo: AppStateRepository,
    pub portal_status_client: LegacyPortalStatusClient,
    pub traffic_service: AccountTrafficService,
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
        let mut current_online_id = success_account
            .map(|account| account.id.clone())
            .unwrap_or_default();
        let panel_accounts = accounts
            .iter()
            .filter(|account| {
                success_account
                    .as_ref()
                    .map_or(true, |current| account.account.id != current.id)
            })
            .cloned()
            .collect::<Vec<_>>();
        let panel_snapshots = self
            .traffic_service
            .fetch_balances_limited(
                &panel_accounts,
                local_ip,
                AccountTrafficService::DEFAULT_PANEL_QUERY_CONCURRENCY,
            )
            .await;
        let mut snapshot_map = AccountTrafficService::to_snapshot_map(panel_snapshots);
        restore_failed_snapshots_from_cache(&mut snapshot_map, &store.cached_traffic_snapshots);
        if let (Some(account), Some(info)) = (success_account, success_info.as_ref()) {
            snapshot_map.insert(
                account.id.clone(),
                build_single_success_snapshot(
                    account,
                    info,
                    store.cached_traffic_snapshots.get(&account.id),
                ),
            );
        }
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

fn restore_failed_snapshots_from_cache(
    snapshots: &mut std::collections::BTreeMap<String, AccountTrafficSnapshot>,
    cached: &std::collections::BTreeMap<String, crate::domain::models::CachedTrafficSnapshot>,
) {
    let restored = restore_cached_snapshots(cached);
    for (account_id, snapshot) in snapshots.iter_mut() {
        if snapshot.status_text != "查询失败" {
            continue;
        }
        let Some(previous) = restored.get(account_id) else {
            continue;
        };
        let failure_detail = snapshot.detail_text.clone();
        let mut previous = previous.clone();
        previous.status_text = "使用缓存".to_string();
        previous.detail_text = format!("本次查询失败：{failure_detail}");
        *snapshot = previous;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::Local;

    use super::restore_failed_snapshots_from_cache;
    use crate::domain::models::{AccountTrafficSnapshot, CachedTrafficSnapshot};

    #[test]
    fn failed_refresh_keeps_previous_success_snapshot() {
        let mut snapshots = BTreeMap::from([(
            "acc-1".to_string(),
            AccountTrafficSnapshot::failed("acc-1", "SSO 返回登录页", Local::now()),
        )]);
        let cached = BTreeMap::from([(
            "acc-1".to_string(),
            CachedTrafficSnapshot {
                used_traffic_text: "12.5G".to_string(),
                product_balance_text: "70.00GB".to_string(),
                included_package_text: String::new(),
                online_device_count_text: "1".to_string(),
                package_text: "免费70GB".to_string(),
                status_text: "已同步".to_string(),
                detail_text: "计费方式：免费70GB".to_string(),
                queried_at: Some(Local::now()),
                progress_percent: Some(0.18),
            },
        )]);

        restore_failed_snapshots_from_cache(&mut snapshots, &cached);

        let snapshot = snapshots.get("acc-1").expect("snapshot restored");
        assert_eq!(snapshot.used_traffic_text, "12.5G");
        assert_eq!(snapshot.status_text, "使用缓存");
        assert!(snapshot.detail_text.contains("SSO 返回登录页"));
    }
}
