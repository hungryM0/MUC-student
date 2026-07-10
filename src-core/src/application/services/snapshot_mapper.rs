use std::collections::BTreeMap;

use chrono::Local;

use crate::application::dto::{
    AccountDto, AppSnapshotDto, LoginStateDto, PoolQuotaDto, PreferenceDto, RefreshStateDto,
};
use crate::application::runtime::AppRuntimeState;
use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::CachedTrafficSnapshot;
use crate::domain::policies::traffic_math::{
    build_pool_quota_summary, normalize_included_package_text,
};

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
                    included_package_text: normalize_included_package_text(
                        &snapshot.included_package_text,
                    ),
                    package_total_text: snapshot.package_total_text.clone(),
                    package_available_text: snapshot.package_available_text.clone(),
                    online_device_count_text: snapshot.online_device_count_text.clone(),
                    package_text: snapshot.package_text.clone(),
                    status_text: snapshot.status_text.clone(),
                    detail_text: snapshot.detail_text.clone(),
                    is_unlimited_plan: snapshot.is_unlimited_plan,
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
                    included_package_text: normalize_included_package_text(
                        &snapshot.included_package_text,
                    ),
                    package_total_text: snapshot.package_total_text.clone(),
                    package_available_text: snapshot.package_available_text.clone(),
                    online_device_count_text: snapshot.online_device_count_text.clone(),
                    package_text: snapshot.package_text.clone(),
                    status_text: snapshot.status_text.clone(),
                    detail_text: snapshot.detail_text.clone(),
                    is_unlimited_plan: snapshot.is_unlimited_plan,
                    queried_at: Some(snapshot.queried_at),
                    progress_percent: snapshot.progress_percent,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::restore_cached_snapshots;
    use crate::domain::models::CachedTrafficSnapshot;

    #[test]
    fn restore_cached_snapshots_normalizes_dirty_included_package_text() {
        let mut cached = BTreeMap::new();
        cached.insert(
            "acc-1".to_string(),
            CachedTrafficSnapshot {
                included_package_text: "含70.00GB套餐流量".to_string(),
                package_total_text: "30.00GB".to_string(),
                package_available_text: "25.883GB".to_string(),
                ..Default::default()
            },
        );
        cached.insert(
            "acc-2".to_string(),
            CachedTrafficSnapshot {
                included_package_text: "含30.00GB增值套餐".to_string(),
                package_total_text: "30.00GB".to_string(),
                package_available_text: "25.883GB".to_string(),
                ..Default::default()
            },
        );

        let snapshots = restore_cached_snapshots(&cached);

        assert_eq!(
            snapshots
                .get("acc-1")
                .expect("snapshot")
                .included_package_text,
            ""
        );
        assert_eq!(
            snapshots
                .get("acc-2")
                .expect("snapshot")
                .included_package_text,
            "含30.00GB套餐流量"
        );
    }
}
