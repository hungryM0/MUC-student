use chrono::Local;

use crate::domain::models::traffic::{AccountTrafficSnapshot, OnlineDeviceRecord};
use crate::domain::models::{CachedTrafficSnapshot, PortalAccount};
use crate::domain::policies::traffic_math::build_progress_percent;
use crate::infrastructure::parsers::legacy_portal_success_page_parser::LegacyPortalSuccessInfo;

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
