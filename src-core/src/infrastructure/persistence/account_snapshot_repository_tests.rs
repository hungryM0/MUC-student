use std::collections::BTreeMap;
use std::sync::Arc;

use tempfile::tempdir;

use crate::domain::models::CachedTrafficSnapshot;
use crate::infrastructure::persistence::account_repository::AccountRepository;
use crate::infrastructure::persistence::account_snapshot_repository::AccountSnapshotRepository;
use crate::infrastructure::persistence::database::AppDatabase;
use crate::infrastructure::persistence::runtime_paths::RuntimePaths;
use crate::infrastructure::security::credential_vault::{CredentialVault, MemoryCredentialVault};

#[test]
fn saves_cached_snapshots_and_status_order() {
    let root = tempdir().expect("create temp dir");
    let paths = RuntimePaths::from_cwd_for_tests(root.path()).expect("create paths");
    let db = AppDatabase::open(&paths).expect("open db");
    let vault: Arc<dyn CredentialVault> = Arc::new(MemoryCredentialVault::default());
    let account_repo = AccountRepository::new(db.clone(), vault);
    let snapshot_repo = AccountSnapshotRepository::new(db);
    let account = account_repo
        .add_account("主号", "20260001", "secret-1")
        .expect("add account");

    let mut snapshots = BTreeMap::new();
    snapshots.insert(
        account.id.clone(),
        CachedTrafficSnapshot {
            used_traffic_text: "1G".to_string(),
            status_text: "已同步".to_string(),
            ..Default::default()
        },
    );
    snapshot_repo
        .save_cached_traffic_snapshots(
            &[account.clone()],
            snapshots,
            account.id.clone(),
            vec![account.id.clone()],
        )
        .expect("save snapshots");

    let state = snapshot_repo
        .load_state_for_accounts(&[account.clone()])
        .expect("load snapshot state");
    assert_eq!(
        state
            .cached_traffic_snapshots
            .get(&account.id)
            .expect("snapshot")
            .used_traffic_text,
        "1G"
    );
    assert_eq!(state.current_online_account_id, account.id.clone());
    assert_eq!(state.status_card_order_snapshot, vec![account.id]);
}
