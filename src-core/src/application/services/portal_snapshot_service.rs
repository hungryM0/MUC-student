use chrono::Local;

use crate::domain::models::traffic::{AccountTrafficSnapshot, OnlineDeviceRecord};
use crate::domain::models::{CachedTrafficSnapshot, PortalAccount};
use crate::domain::policies::traffic_math::{
    build_progress_percent, extract_total_quota_from_billing_policy, format_traffic_bytes_as_gb,
    format_traffic_text_as_gb,
};
use crate::infrastructure::parsers::legacy_portal_online_info_parser::LegacyPortalOnlineInfo;
use crate::infrastructure::parsers::legacy_portal_success_page_parser::LegacyPortalSuccessInfo;

pub fn build_single_success_snapshot(
    account: &PortalAccount,
    info: &LegacyPortalSuccessInfo,
    cached_current: Option<&CachedTrafficSnapshot>,
) -> AccountTrafficSnapshot {
    build_single_success_snapshot_with_online_info(account, info, None, cached_current)
}

pub fn build_single_success_snapshot_with_online_info(
    account: &PortalAccount,
    info: &LegacyPortalSuccessInfo,
    online_info: Option<&LegacyPortalOnlineInfo>,
    cached_current: Option<&CachedTrafficSnapshot>,
) -> AccountTrafficSnapshot {
    let product_balance_text = extract_total_quota_from_billing_policy(&info.billing_policy)
        .or_else(|| {
            cached_current
                .map(|item| item.product_balance_text.clone())
                .filter(|text| !text.trim().is_empty())
        })
        .unwrap_or_else(|| "-".to_string());
    let used_traffic_text = online_info
        .and_then(|info| format_traffic_bytes_as_gb(&info.used_traffic_bytes))
        .unwrap_or_else(|| format_traffic_text_as_gb(&info.used_traffic));
    let included_package_text = cached_current
        .map(|item| item.included_package_text.clone())
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| {
            info.paid_package_quota
                .as_ref()
                .map(|quota| format!("含{quota}套餐流量"))
                .unwrap_or_default()
        });
    let matched_local_ip_device = Some(OnlineDeviceRecord {
        ip: info.ip.clone(),
        device_id: String::new(),
        logout_path: String::new(),
    });
    AccountTrafficSnapshot {
        account_id: account.id.clone(),
        used_traffic_text: used_traffic_text.clone(),
        product_balance_text: product_balance_text.clone(),
        included_package_text,
        package_total_text: String::new(),
        package_available_text: String::new(),
        online_device_count_text: cached_current
            .map(|item| item.online_device_count_text.clone())
            .filter(|text| !text.trim().is_empty())
            .unwrap_or_else(|| "-".to_string()),
        package_text: cached_current
            .map(|item| item.package_text.clone())
            .filter(|text| !text.trim().is_empty())
            .unwrap_or_else(|| "-".to_string()),
        status_text: "成功页同步".to_string(),
        detail_text: format!("计费方式：{}", info.billing_policy),
        queried_at: Local::now(),
        online_devices: Vec::new(),
        matched_local_ip_device,
        progress_percent: build_progress_percent(&used_traffic_text, &product_balance_text),
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
    use super::{
        build_single_success_snapshot, build_single_success_snapshot_with_online_info,
        username_matches,
    };
    use crate::domain::models::PortalAccount;
    use crate::infrastructure::parsers::legacy_portal_online_info_parser::LegacyPortalOnlineInfo;
    use crate::infrastructure::parsers::legacy_portal_success_page_parser::LegacyPortalSuccessInfo;

    #[test]
    fn username_match_accepts_provider_suffix_difference() {
        assert!(username_matches("13377235977@deep", "13377235977"));
        assert!(username_matches("13377235977", "13377235977@deep"));
        assert!(!username_matches("13377235978@deep", "13377235977"));
    }

    #[test]
    fn current_account_snapshot_uses_billing_policy_quota() {
        let account = PortalAccount {
            id: "acc-1".to_string(),
            remark_name: "当前账号".to_string(),
            username: "25011777".to_string(),
        };
        let info = LegacyPortalSuccessInfo {
            ip: "10.0.0.1".to_string(),
            username: "25011777".to_string(),
            used_traffic: "22,230.78M".to_string(),
            billing_policy: "免费70GB".to_string(),
            paid_package_quota: None,
        };

        let snapshot = build_single_success_snapshot(&account, &info, None);

        assert_eq!(snapshot.used_traffic_text, "21.71GB");
        assert_eq!(snapshot.product_balance_text, "70.00GB");
        assert_eq!(snapshot.progress_percent, Some(31.0));
        assert_eq!(snapshot.included_package_text, "");
    }

    #[test]
    fn success_page_fallback_does_not_fake_online_device_count() {
        let account = PortalAccount {
            id: "acc-1".to_string(),
            remark_name: "当前账号".to_string(),
            username: "25011777".to_string(),
        };
        let info = LegacyPortalSuccessInfo {
            ip: "10.0.0.1".to_string(),
            username: "25011777".to_string(),
            used_traffic: "22,230.78M".to_string(),
            billing_policy: "免费70GB".to_string(),
            paid_package_quota: None,
        };

        let snapshot = build_single_success_snapshot(&account, &info, None);

        assert_eq!(snapshot.online_device_count_text, "-");
        assert_eq!(snapshot.status_text, "成功页同步");
        assert!(snapshot.online_devices.is_empty());
        assert_eq!(
            snapshot
                .matched_local_ip_device
                .as_ref()
                .map(|device| device.ip.as_str()),
            Some("10.0.0.1")
        );
    }

    #[test]
    fn success_snapshot_prefers_online_info_used_traffic_bytes() {
        let account = PortalAccount {
            id: "acc-1".to_string(),
            remark_name: "当前账号".to_string(),
            username: "25011777".to_string(),
        };
        let success_info = LegacyPortalSuccessInfo {
            ip: "10.0.0.1".to_string(),
            username: "25011777".to_string(),
            used_traffic: "0M".to_string(),
            billing_policy: "免费70GB".to_string(),
            paid_package_quota: None,
        };
        let online_info = LegacyPortalOnlineInfo {
            used_traffic_bytes: "41675360258".to_string(),
            online_seconds: "2242187".to_string(),
            balance: "0.00".to_string(),
            mac: "c4:0f:08:8e:0e:5e".to_string(),
            billing_state: "0".to_string(),
            ip: "10.0.0.1".to_string(),
        };

        let snapshot = build_single_success_snapshot_with_online_info(
            &account,
            &success_info,
            Some(&online_info),
            None,
        );

        assert_eq!(snapshot.used_traffic_text, "38.81GB");
        assert_eq!(snapshot.progress_percent, Some(55.4));
    }

    #[test]
    fn current_account_snapshot_uses_paid_package_quota_when_available() {
        let account = PortalAccount {
            id: "acc-1".to_string(),
            remark_name: "当前账号".to_string(),
            username: "25011777".to_string(),
        };
        let info = LegacyPortalSuccessInfo {
            ip: "10.0.0.1".to_string(),
            username: "25011777".to_string(),
            used_traffic: "101,264.17M".to_string(),
            billing_policy: "Package-use-20元30GB".to_string(),
            paid_package_quota: Some("30.00GB".to_string()),
        };

        let snapshot = build_single_success_snapshot(&account, &info, None);

        assert_eq!(snapshot.included_package_text, "含30.00GB套餐流量");
    }
}
