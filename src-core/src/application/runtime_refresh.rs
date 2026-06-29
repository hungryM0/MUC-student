use std::collections::HashSet;

use crate::application::error::AppResult;
use crate::application::runtime::SharedRuntimeState;
use crate::application::services::snapshot_mapper::restore_cached_snapshots;
use crate::infrastructure::persistence::account_repository::AccountRepository;
use crate::infrastructure::persistence::app_state_repository::AppStateRepository;

pub fn refresh_runtime_from_disk(
    state: &SharedRuntimeState,
    account_repo: &AccountRepository,
    app_state_repo: &AppStateRepository,
) -> AppResult<()> {
    let account_store = account_repo.load_store()?;
    let app_state = app_state_repo.load_state()?;
    let preferences = app_state_repo.load_preferences()?;
    let valid_ids = account_store
        .accounts
        .iter()
        .map(|account| account.id.clone())
        .collect::<HashSet<_>>();
    let mut runtime = state.write();
    runtime.current_online_account_id = account_store.current_online_account_id.clone();
    runtime.snapshots.retain(|id, _| valid_ids.contains(id));
    for (id, snapshot) in restore_cached_snapshots(&account_store.cached_traffic_snapshots) {
        runtime.snapshots.entry(id).or_insert(snapshot);
    }
    runtime.account_store = account_store;
    runtime.app_state = app_state;
    runtime.preferences = preferences;
    Ok(())
}
