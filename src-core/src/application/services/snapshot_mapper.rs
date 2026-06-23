use std::collections::BTreeMap;

use chrono::Local;

use crate::application::dto::{
    AccountDto, AppSnapshotDto, LoginStateDto, PoolQuotaDto, PreferenceDto, RefreshStateDto,
};
use crate::application::runtime::AppRuntimeState;
use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::CachedTrafficSnapshot;
use crate::domain::policies::traffic_math::build_pool_quota_summary;

pub fn build_app_snapshot(state: &AppRuntimeState) -> AppSnapshotDto {
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
    AppSnapshotDto {
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
    }
}

pub fn restore_cached_snapshots(
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

pub fn to_cached_snapshots(
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
