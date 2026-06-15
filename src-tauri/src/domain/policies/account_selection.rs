use crate::domain::models::traffic::AccountTrafficSnapshot;
use crate::domain::models::PortalAccount;

pub fn find_current_online_account(
    accounts: &[PortalAccount],
    snapshots: &[AccountTrafficSnapshot],
) -> Option<PortalAccount> {
    let snapshot_map: std::collections::HashMap<&str, &AccountTrafficSnapshot> = snapshots
        .iter()
        .map(|snapshot| (snapshot.account_id.as_str(), snapshot))
        .collect();
    accounts
        .iter()
        .find(|account| {
            snapshot_map
                .get(account.id.as_str())
                .and_then(|snapshot| snapshot.matched_local_ip_device.as_ref())
                .is_some()
        })
        .cloned()
}
