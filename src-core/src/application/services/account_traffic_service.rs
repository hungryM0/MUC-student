use std::collections::BTreeMap;

use chrono::Local;

use crate::application::error::AppResult;
use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::policies::traffic_math::build_progress_percent;
use crate::infrastructure::network::self_service_panel_client::SelfServicePanelClient;
use crate::infrastructure::parsers::panel_home_parser::parse_panel_home;
use crate::infrastructure::persistence::account_repository::AccountWithPassword;

#[derive(Clone)]
pub struct AccountTrafficService {
    panel_client: SelfServicePanelClient,
}

impl AccountTrafficService {
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

    pub async fn fetch_balance(
        &self,
        account: &AccountWithPassword,
        local_ip: Option<&str>,
    ) -> AppResult<AccountTrafficSnapshot> {
        let home_html = self
            .panel_client
            .fetch_authenticated_html(account, "/home")
            .await?;
        let (
            package_name,
            billing_policy,
            used_traffic,
            product_balance,
            online_devices,
            matched_local,
        ) = parse_panel_home(&home_html, local_ip)?;
        Ok(AccountTrafficSnapshot {
            account_id: account.account.id.clone(),
            used_traffic_text: used_traffic.clone(),
            product_balance_text: product_balance.clone(),
            included_package_text:
                crate::infrastructure::parsers::panel_home_parser::build_product_balance_texts(
                    &home_html,
                )
                .1,
            online_device_count_text: online_devices.len().to_string(),
            package_text: package_name,
            status_text: "已同步".to_string(),
            detail_text: format!("计费策略：{billing_policy}"),
            queried_at: Local::now(),
            online_devices,
            matched_local_ip_device: matched_local,
            progress_percent: build_progress_percent(&used_traffic, &product_balance),
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
