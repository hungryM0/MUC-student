use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::Local;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::application::error::AppResult;
use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::PortalAccount;
use crate::domain::policies::traffic_math::{build_progress_percent, format_traffic_text_as_gb};
use crate::infrastructure::network::self_service_panel_client::SelfServicePanelClient;
use crate::infrastructure::parsers::panel_home_parser::{
    build_product_balance_texts, parse_panel_home,
};
use crate::infrastructure::persistence::account_repository::AccountWithPassword;

#[derive(Clone)]
pub struct AccountTrafficService {
    panel_client: SelfServicePanelClient,
}

impl AccountTrafficService {
    pub const DEFAULT_PANEL_QUERY_CONCURRENCY: usize = 2;

    pub fn new(panel_client: SelfServicePanelClient) -> Self {
        Self { panel_client }
    }

    pub async fn fetch_balances(
        &self,
        accounts: &[AccountWithPassword],
        local_ip: Option<&str>,
    ) -> Vec<AccountTrafficSnapshot> {
        let mut snapshots = Vec::new();
        for account in accounts {
            match self.fetch_balance(account, local_ip).await {
                Ok(snapshot) => snapshots.push(snapshot),
                Err(err) => snapshots.push(AccountTrafficSnapshot::failed(
                    account.account.id.clone(),
                    err.to_string(),
                    Local::now(),
                )),
            }
        }
        snapshots
    }

    pub async fn fetch_balances_limited(
        &self,
        accounts: &[AccountWithPassword],
        local_ip: Option<&str>,
        max_concurrent: usize,
    ) -> Vec<AccountTrafficSnapshot> {
        let limit = max_concurrent.max(1);
        let semaphore = Arc::new(Semaphore::new(limit));
        let local_ip = local_ip.map(str::to_string);
        let mut join_set = JoinSet::new();

        for account in accounts.iter().cloned() {
            let service = self.clone();
            let semaphore = semaphore.clone();
            let local_ip = local_ip.clone();
            join_set.spawn(async move {
                let account_id = account.account.id.clone();
                let Ok(_permit) = semaphore.acquire_owned().await else {
                    return AccountTrafficSnapshot::failed(
                        account_id,
                        "面板查询队列已关闭",
                        Local::now(),
                    );
                };
                match service.fetch_balance(&account, local_ip.as_deref()).await {
                    Ok(snapshot) => snapshot,
                    Err(err) => AccountTrafficSnapshot::failed(
                        account.account.id.clone(),
                        err.to_string(),
                        Local::now(),
                    ),
                }
            });
        }

        let mut snapshots = Vec::new();
        while let Some(result) = join_set.join_next().await {
            if let Ok(snapshot) = result {
                snapshots.push(snapshot);
            }
        }
        snapshots
    }

    pub async fn fetch_balance(
        &self,
        account: &AccountWithPassword,
        local_ip: Option<&str>,
    ) -> AppResult<AccountTrafficSnapshot> {
        let home_html = self
            .panel_client
            .fetch_authenticated_html(account, "/home")
            .await?;
        Self::snapshot_from_panel_home(&account.account, &home_html, local_ip)
    }

    pub fn snapshot_from_panel_home(
        account: &PortalAccount,
        home_html: &str,
        local_ip: Option<&str>,
    ) -> AppResult<AccountTrafficSnapshot> {
        let panel_home = parse_panel_home(home_html, local_ip)?;
        let used_traffic_text = format_traffic_text_as_gb(&panel_home.used_traffic);
        Ok(AccountTrafficSnapshot {
            account_id: account.id.clone(),
            used_traffic_text: used_traffic_text.clone(),
            product_balance_text: panel_home.product_balance.clone(),
            included_package_text: build_product_balance_texts(home_html).1,
            online_device_count_text: panel_home.online_devices.len().to_string(),
            package_text: panel_home.package_name,
            status_text: "已同步".to_string(),
            detail_text: format!("计费策略：{}", panel_home.billing_policy),
            queried_at: Local::now(),
            online_devices: panel_home.online_devices,
            matched_local_ip_device: panel_home.matched_local_ip_device,
            progress_percent: build_progress_percent(
                &used_traffic_text,
                &panel_home.product_balance,
            ),
        })
    }

    pub fn to_snapshot_map(
        snapshots: Vec<AccountTrafficSnapshot>,
    ) -> BTreeMap<String, AccountTrafficSnapshot> {
        snapshots
            .into_iter()
            .map(|snapshot| (snapshot.account_id.clone(), snapshot))
            .collect()
    }
}
