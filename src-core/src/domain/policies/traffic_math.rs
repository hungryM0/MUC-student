use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::{AccountStore, PortalAccount};

#[derive(Clone, Debug, Default)]
pub struct RuntimeFlags {
    pub login_running: bool,
    pub refresh_running: bool,
    pub logout_running: bool,
    pub current_ip: String,
    pub current_online_account_id: String,
}

pub fn build_auto_switch_candidate(
    account_store: &AccountStore,
    snapshots: &BTreeMap<String, AccountTrafficSnapshot>,
    recent_account_ids: &[String],
) -> Option<PortalAccount> {
    let current_account = account_store
        .accounts
        .iter()
        .find(|account| account.id == account_store.selected_account_id)?;

    let current_snapshot = snapshots.get(&current_account.id)?;
    if current_snapshot.progress_percent? < 100.0 {
        return None;
    }

    let mut candidates: Vec<&PortalAccount> = account_store
        .accounts
        .iter()
        .filter(|account| account.id != current_account.id)
        .filter(|account| {
            snapshots
                .get(&account.id)
                .and_then(|snapshot| snapshot.progress_percent)
                .is_some_and(|percent| percent < 100.0)
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    let rank_map: HashMap<&str, usize> = recent_account_ids
        .iter()
        .enumerate()
        .map(|(idx, id)| (id.as_str(), idx))
        .collect();

    candidates.sort_by_key(|account| {
        (
            rank_map.get(account.id.as_str()).copied().unwrap_or(10_000),
            account_store
                .accounts
                .iter()
                .position(|item| item.id == account.id)
                .unwrap_or(usize::MAX),
        )
    });

    candidates.into_iter().next().cloned()
}

pub fn build_status_card_order(
    account_store: &AccountStore,
    snapshots: &BTreeMap<String, AccountTrafficSnapshot>,
    current_online_account_id: &str,
    order_snapshot: &[String],
) -> Vec<String> {
    let order_index: HashMap<&str, usize> = order_snapshot
        .iter()
        .enumerate()
        .map(|(idx, id)| (id.as_str(), idx))
        .collect();

    let mut other_accounts: Vec<&PortalAccount> = account_store
        .accounts
        .iter()
        .filter(|account| account.id != current_online_account_id)
        .collect();

    other_accounts.sort_by(|left, right| {
        let left_progress = snapshots
            .get(&left.id)
            .and_then(|snapshot| snapshot.progress_percent);
        let right_progress = snapshots
            .get(&right.id)
            .and_then(|snapshot| snapshot.progress_percent);
        match (left_progress, right_progress) {
            (Some(a), Some(b)) => b.partial_cmp(&a).unwrap_or(Ordering::Equal),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => order_index
                .get(left.id.as_str())
                .copied()
                .unwrap_or(usize::MAX)
                .cmp(
                    &order_index
                        .get(right.id.as_str())
                        .copied()
                        .unwrap_or(usize::MAX),
                ),
        }
        .then_with(|| {
            order_index
                .get(left.id.as_str())
                .copied()
                .unwrap_or(usize::MAX)
                .cmp(
                    &order_index
                        .get(right.id.as_str())
                        .copied()
                        .unwrap_or(usize::MAX),
                )
        })
    });

    let mut final_order = Vec::new();
    if !current_online_account_id.is_empty()
        && account_store
            .accounts
            .iter()
            .any(|account| account.id == current_online_account_id)
    {
        final_order.push(current_online_account_id.to_string());
    }
    final_order.extend(other_accounts.into_iter().map(|account| account.id.clone()));
    final_order
}

pub fn build_pool_quota_summary(
    account_store: &AccountStore,
    snapshots: &BTreeMap<String, AccountTrafficSnapshot>,
) -> (String, String, String, Option<f64>) {
    let mut used_total_mb = 0.0;
    let mut total_balance_gb = 0.0;
    let mut included_package_total_mb = 0.0;
    let mut has_used_value = false;
    let mut has_total_value = false;

    for account in &account_store.accounts {
        let used_text = snapshots
            .get(&account.id)
            .map(|item| item.used_traffic_text.as_str())
            .or_else(|| {
                account_store
                    .cached_traffic_snapshots
                    .get(&account.id)
                    .map(|item| item.used_traffic_text.as_str())
            });
        let total_text = snapshots
            .get(&account.id)
            .map(|item| item.product_balance_text.as_str())
            .or_else(|| {
                account_store
                    .cached_traffic_snapshots
                    .get(&account.id)
                    .map(|item| item.product_balance_text.as_str())
            });
        let included_text = snapshots
            .get(&account.id)
            .map(|item| item.included_package_text.as_str())
            .or_else(|| {
                account_store
                    .cached_traffic_snapshots
                    .get(&account.id)
                    .map(|item| item.included_package_text.as_str())
            });

        if let Some(used_mb) = used_text.and_then(parse_traffic_text_to_mb) {
            used_total_mb += used_mb;
            has_used_value = true;
        }
        if let Some(total_gb) = total_text.and_then(parse_traffic_text_to_gb) {
            total_balance_gb += total_gb;
            has_total_value = true;
        }
        if let Some(included_gb) = included_text.and_then(extract_included_package_gb) {
            included_package_total_mb += included_gb * 1024.0;
        }
    }

    let used_text = if has_used_value {
        format_gigabytes(used_total_mb)
    } else {
        "-".to_string()
    };
    let total_text = if has_total_value {
        format!("{total_balance_gb:.2}GB")
    } else {
        "-".to_string()
    };
    let included_text = if has_total_value && included_package_total_mb > 0.0 {
        format!("含{:.2}GB套餐流量", included_package_total_mb / 1024.0)
    } else {
        String::new()
    };

    let progress = if has_total_value && has_used_value && total_balance_gb > 0.0 {
        Some(round_to(
            ((used_total_mb / (total_balance_gb * 1024.0)) * 100.0).clamp(0.0, 100.0),
            1,
        ))
    } else {
        None
    };

    (used_text, total_text, included_text, progress)
}

pub fn build_progress_percent(used_traffic_text: &str, total_traffic_text: &str) -> Option<f64> {
    let used_mb = parse_traffic_text_to_mb(used_traffic_text)?;
    let total_mb = parse_traffic_text_to_mb(total_traffic_text)?;
    if total_mb <= 0.0 {
        return None;
    }
    Some(round_to(
        ((used_mb / total_mb) * 100.0).clamp(0.0, 100.0),
        1,
    ))
}

pub fn build_remaining_traffic_text(
    total_traffic_text: &str,
    used_traffic_text: &str,
) -> Option<String> {
    let total_mb = parse_traffic_text_to_mb(total_traffic_text)?;
    let used_mb = parse_traffic_text_to_mb(used_traffic_text)?;
    Some(format_gigabytes((total_mb - used_mb).max(0.0)))
}

pub fn parse_traffic_text_to_mb(text: &str) -> Option<f64> {
    let normalized = text.trim().to_uppercase().replace([' ', ','], "");
    let re = regex::Regex::new(r"(\d+(?:\.\d+)?)(K|M|G|T|B)(?:YTE|YTES|B)?").ok()?;
    let caps = re.captures(&normalized)?;
    let value: f64 = caps.get(1)?.as_str().parse().ok()?;
    match caps.get(2)?.as_str() {
        "B" => Some(value / 1024.0 / 1024.0),
        "K" => Some(value / 1024.0),
        "M" => Some(value),
        "G" => Some(value * 1024.0),
        "T" => Some(value * 1024.0 * 1024.0),
        _ => None,
    }
}

pub fn parse_traffic_text_to_gb(text: &str) -> Option<f64> {
    parse_traffic_text_to_mb(text).map(|mb| mb / 1024.0)
}

pub fn extract_total_quota_from_billing_policy(text: &str) -> Option<String> {
    let normalized = text.replace([' ', ','], "");
    let free_re = regex::Regex::new(r"免费([0-9]+(?:\.[0-9]+)?)GB").ok()?;
    if let Some(caps) = free_re.captures(&normalized) {
        let value = caps.get(1)?.as_str().parse::<f64>().ok()?;
        return Some(format!("{value:.2}GB"));
    }

    let quota_re = regex::Regex::new(r"([0-9]+(?:\.[0-9]+)?)GB").ok()?;
    quota_re
        .captures_iter(&normalized)
        .filter_map(|caps| caps.get(1)?.as_str().parse::<f64>().ok())
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
        .map(|value| format!("{value:.2}GB"))
}

pub fn extract_paid_package_quota_from_billing_policy(text: &str) -> Option<String> {
    let normalized = text.replace([' ', ','], "");
    let explicit_patterns = [
        r"Package(?:-use)?-[^/]*?([0-9]+(?:\.[0-9]+)?)GB",
        r"套餐[^/]*?([0-9]+(?:\.[0-9]+)?)GB",
        r"包月[^/]*?([0-9]+(?:\.[0-9]+)?)GB",
    ];

    explicit_patterns.iter().find_map(|pattern| {
        regex::Regex::new(pattern)
            .ok()?
            .captures(&normalized)
            .and_then(|caps| caps.get(1))
            .and_then(|item| item.as_str().parse::<f64>().ok())
            .map(|value| format!("{value:.2}GB"))
    })
}

pub fn normalize_included_package_text(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Some(included_gb) = extract_included_package_gb(trimmed) else {
        return trimmed.to_string();
    };
    if (included_gb - 70.0).abs() < 0.01 {
        return String::new();
    }
    format!("含{included_gb:.2}GB套餐流量")
}

pub fn format_traffic_text_as_gb(text: &str) -> String {
    parse_traffic_text_to_mb(text)
        .map(format_gigabytes)
        .unwrap_or_else(|| text.trim().replace(',', ""))
}

pub fn format_traffic_bytes_as_gb(text: &str) -> Option<String> {
    let bytes = text.trim().replace(',', "").parse::<f64>().ok()?;
    Some(format_gigabytes(bytes / 1024.0 / 1024.0))
}

pub fn extract_included_package_gb(text: &str) -> Option<f64> {
    let normalized = text.replace(' ', "");
    let re = regex::Regex::new(r"含([0-9]+(?:\.[0-9]+)?)GB(?:套餐流量|增值套餐)").ok()?;
    let caps = re.captures(&normalized)?;
    caps.get(1)?.as_str().parse().ok()
}

pub fn format_megabytes(value_mb: f64) -> String {
    let mb = value_mb.max(0.0);
    if mb >= 1024.0 * 1024.0 {
        format!("{:.2}T", mb / 1024.0 / 1024.0)
    } else if mb >= 1024.0 {
        format!("{:.2}G", mb / 1024.0)
    } else if mb >= 1.0 {
        format!("{:.2}M", mb)
    } else {
        format!("{:.2}K", mb * 1024.0)
    }
}

pub fn format_gigabytes(value_mb: f64) -> String {
    format!("{:.2}GB", value_mb.max(0.0) / 1024.0)
}

fn round_to(value: f64, digits: i32) -> f64 {
    let factor = 10f64.powi(digits.max(0));
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_progress_and_remaining_text() {
        assert_eq!(build_progress_percent("1GB", "2GB"), Some(50.0));
        assert_eq!(
            build_remaining_traffic_text("2GB", "512MB"),
            Some("1.50GB".to_string())
        );
    }

    #[test]
    fn parses_comma_traffic_and_formats_gb() {
        assert_eq!(
            format_traffic_text_as_gb("22,230.78M"),
            "21.71GB".to_string()
        );
        assert_eq!(build_progress_percent("22,230.78M", "70GB"), Some(31.0));
    }

    #[test]
    fn formats_traffic_bytes_as_gb() {
        assert_eq!(
            format_traffic_bytes_as_gb("41675360258"),
            Some("38.81GB".to_string())
        );
    }

    #[test]
    fn extracts_total_quota_from_billing_policy() {
        assert_eq!(
            extract_total_quota_from_billing_policy("免费70GB"),
            Some("70.00GB".to_string())
        );
        assert_eq!(
            extract_total_quota_from_billing_policy("免费45GB/1GB1元（超出45GB）/校内流量"),
            Some("45.00GB".to_string())
        );
    }

    #[test]
    fn extracts_paid_package_quota_from_billing_policy() {
        assert_eq!(
            extract_paid_package_quota_from_billing_policy("Package-use-20元30GB"),
            Some("30.00GB".to_string())
        );
        assert_eq!(
            extract_paid_package_quota_from_billing_policy("免费70GB/Package-use-20元30GB"),
            Some("30.00GB".to_string())
        );
        assert_eq!(
            extract_paid_package_quota_from_billing_policy("免费70GB"),
            None
        );
        assert_eq!(
            extract_paid_package_quota_from_billing_policy("免费70GB/1GB1元（超出70GB）/校内流量"),
            None
        );
    }

    #[test]
    fn extracts_included_package_gb_for_both_text_styles() {
        assert_eq!(extract_included_package_gb("含30.00GB套餐流量"), Some(30.0));
        assert_eq!(extract_included_package_gb("含30.00GB增值套餐"), Some(30.0));
    }

    #[test]
    fn normalizes_included_package_text_and_drops_fake_free_quota() {
        assert_eq!(
            normalize_included_package_text("含30.00GB增值套餐"),
            "含30.00GB套餐流量"
        );
        assert_eq!(normalize_included_package_text("含70.00GB套餐流量"), "");
        assert_eq!(normalize_included_package_text(""), "");
    }
}
