use std::collections::BTreeMap;

use chrono::Local;
use tokio::task::JoinSet;

use crate::application::error::{AppError, AppResult};
use crate::domain::models::traffic::{AccountTrafficSnapshot, OnlineDeviceRecord};
use crate::domain::models::{CachedTrafficSnapshot, PortalAccount};
use crate::domain::policies::traffic_math::build_progress_percent;
use crate::infrastructure::network::legacy_portal_auth_client::LegacyPortalAuthClient;
use crate::infrastructure::network::legacy_portal_status_client::LegacyPortalStatusClient;
use crate::infrastructure::parsers::legacy_portal_success_page_parser::LegacyPortalSuccessInfo;
use crate::infrastructure::persistence::account_repository::{
    AccountRepository, AccountWithPassword,
};

#[derive(Clone)]
pub struct PortalSnapshotService {
    account_repo: AccountRepository,
    auth_client: LegacyPortalAuthClient,
    status_client: LegacyPortalStatusClient,
}

impl PortalSnapshotService {
    pub fn new(
        account_repo: AccountRepository,
        auth_client: LegacyPortalAuthClient,
        status_client: LegacyPortalStatusClient,
    ) -> Self {
        Self {
            account_repo,
            auth_client,
            status_client,
        }
    }

    pub async fn fetch_balances_with_probe(
        &self,
        accounts: &[PortalAccount],
        cached: &BTreeMap<String, CachedTrafficSnapshot>,
        current_account: AccountWithPassword,
    ) -> AppResult<BTreeMap<String, AccountTrafficSnapshot>> {
        let parallel = self
            .probe_balances_parallel(accounts, cached, current_account.clone())
            .await;
        match parallel {
            Ok((snapshot_map, collision_detected)) if !collision_detected => {
                self.restore_account(accounts, &current_account).await;
                Ok(snapshot_map)
            }
            _ => {
                self.restore_account(accounts, &current_account).await;
                self.fetch_balances_serial(accounts, cached, current_account)
                    .await
            }
        }
    }

    pub async fn detect_current_account(
        &self,
        accounts: &[PortalAccount],
    ) -> Option<AccountWithPassword> {
        let info = self.status_client.fetch_success_info().await.ok()?;
        let account = accounts
            .iter()
            .find(|account| username_matches(&account.username, &info.username))?;
        self.account_repo.load_account_with_password(account).ok()
    }

    pub async fn restore_account(
        &self,
        accounts: &[PortalAccount],
        target_account: &AccountWithPassword,
    ) {
        let Some(active_account) = self.detect_current_account(accounts).await else {
            let _ = self.auth_client.verify_login(target_account).await;
            return;
        };
        if active_account.account.id == target_account.account.id {
            return;
        }
        let _ = self.auth_client.login_target_account(target_account).await;
    }

    async fn probe_balances_parallel(
        &self,
        accounts: &[PortalAccount],
        cached: &BTreeMap<String, CachedTrafficSnapshot>,
        current_account: AccountWithPassword,
    ) -> AppResult<(BTreeMap<String, AccountTrafficSnapshot>, bool)> {
        let mut join_set = JoinSet::new();
        for account in accounts {
            let auth_client = self.auth_client.clone();
            let status_client = self.status_client.clone();
            let current_account = current_account.clone();
            let target = self.account_repo.load_account_with_password(account)?;
            let cached_current = cached.get(&target.account.id).cloned();
            join_set.spawn(async move {
                let snapshot = if target.account.id == current_account.account.id {
                    let info = status_client.fetch_success_info().await?;
                    require_success_info_matches(&target.account, &info)?;
                    build_single_success_snapshot(&target.account, &info, cached_current.as_ref())
                } else {
                    let _ = auth_client.login_target_account(&target).await?;
                    let info = status_client.fetch_success_info().await?;
                    require_success_info_matches(&target.account, &info)?;
                    build_single_success_snapshot(&target.account, &info, cached_current.as_ref())
                };
                Ok::<_, AppError>((target.account.id.clone(), snapshot))
            });
        }

        let mut snapshots = BTreeMap::new();
        let mut collision_detected = false;
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok((account_id, snapshot))) => {
                    snapshots.insert(account_id, snapshot);
                }
                Ok(Err(err)) => {
                    let _ = err;
                    collision_detected = true;
                }
                Err(err) => {
                    collision_detected = true;
                    let _ = err;
                }
            }
        }

        if snapshots.is_empty() {
            return Err(AppError::Network(
                "并发 portal 探测没有拿到任何账号结果".to_string(),
            ));
        }

        Ok((snapshots, collision_detected))
    }

    async fn fetch_balances_serial(
        &self,
        accounts: &[PortalAccount],
        cached: &BTreeMap<String, CachedTrafficSnapshot>,
        current_account: AccountWithPassword,
    ) -> AppResult<BTreeMap<String, AccountTrafficSnapshot>> {
        let mut snapshots = BTreeMap::new();
        let mut active_account = current_account.clone();
        for account in accounts {
            let target = self.account_repo.load_account_with_password(account)?;
            let cached_current = cached.get(&target.account.id);
            let info_result = if target.account.id == active_account.account.id {
                self.status_client.fetch_success_info().await
            } else {
                match self.auth_client.login_target_account(&target).await {
                    Ok(_) => {
                        active_account = target.clone();
                        self.status_client.fetch_success_info().await
                    }
                    Err(err) => Err(err),
                }
            };
            let info = match info_result {
                Ok(info) => info,
                Err(err) => {
                    self.restore_account(accounts, &current_account).await;
                    return Err(err);
                }
            };
            if let Err(err) = require_success_info_matches(&target.account, &info) {
                self.restore_account(accounts, &current_account).await;
                return Err(err);
            }
            snapshots.insert(
                target.account.id.clone(),
                build_single_success_snapshot(&target.account, &info, cached_current),
            );
        }
        self.restore_account(accounts, &current_account).await;
        Ok(snapshots)
    }
}

pub fn build_single_success_snapshot(
    account: &PortalAccount,
    info: &LegacyPortalSuccessInfo,
    cached_current: Option<&CachedTrafficSnapshot>,
) -> AccountTrafficSnapshot {
    let product_balance_text = cached_current
        .map(|item| item.product_balance_text.clone())
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| "-".to_string());
    let matched_local_ip_device = Some(OnlineDeviceRecord {
        ip: info.ip.clone(),
        device_id: String::new(),
        logout_path: String::new(),
    });
    AccountTrafficSnapshot {
        account_id: account.id.clone(),
        used_traffic_text: info.used_traffic.clone(),
        product_balance_text: product_balance_text.clone(),
        included_package_text: cached_current
            .map(|item| item.included_package_text.clone())
            .unwrap_or_default(),
        online_device_count_text: "1".to_string(),
        package_text: cached_current
            .map(|item| item.package_text.clone())
            .filter(|text| !text.trim().is_empty())
            .unwrap_or_else(|| "-".to_string()),
        status_text: "已同步".to_string(),
        detail_text: format!("计费方式：{}", info.billing_policy),
        queried_at: Local::now(),
        online_devices: matched_local_ip_device.clone().into_iter().collect(),
        matched_local_ip_device,
        progress_percent: build_progress_percent(&info.used_traffic, &product_balance_text),
    }
}

pub fn username_matches(stored: &str, online: &str) -> bool {
    let stored = stored.trim();
    let online = online.trim();
    if stored.eq_ignore_ascii_case(online) {
        return true;
    }
    let stored_base = stored.split('@').next().unwrap_or(stored).trim();
    let online_base = online.split('@').next().unwrap_or(online).trim();
    !stored_base.is_empty() && stored_base.eq_ignore_ascii_case(online_base)
}

fn require_success_info_matches(
    account: &PortalAccount,
    info: &LegacyPortalSuccessInfo,
) -> AppResult<()> {
    if username_matches(&account.username, &info.username) {
        Ok(())
    } else {
        Err(AppError::Network(format!(
            "Portal 返回账号不匹配：期望 {}，实际 {}",
            account.username, info.username
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::username_matches;

    #[test]
    fn username_match_accepts_provider_suffix_difference() {
        assert!(username_matches("13377235977@deep", "13377235977"));
        assert!(username_matches("13377235977", "13377235977@deep"));
        assert!(!username_matches("13377235978@deep", "13377235977"));
    }
}
