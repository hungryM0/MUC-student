use chrono::Local;

use crate::application::error::AppResult;
use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::PortalAccount;
use crate::domain::policies::traffic_math::{build_progress_percent, format_traffic_text_as_gb};
use crate::infrastructure::parsers::panel_home_parser::parse_panel_home;

pub struct AccountTrafficService;

impl AccountTrafficService {
    pub const DEFAULT_PANEL_QUERY_CONCURRENCY: usize = 2;

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
            included_package_text: panel_home.included_package_text,
            package_total_text: panel_home.package_total_text,
            package_available_text: panel_home.package_available_text,
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
}
