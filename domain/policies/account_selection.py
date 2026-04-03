from __future__ import annotations

from domain.models.account import AccountStore, PortalAccount
from domain.models.traffic import AccountTrafficSnapshot


def find_current_online_account(
    accounts: list[PortalAccount],
    snapshots: list[AccountTrafficSnapshot],
) -> PortalAccount | None:
    snapshot_map = {snapshot.account_id: snapshot for snapshot in snapshots}
    for account in accounts:
        snapshot = snapshot_map.get(account.id)
        if snapshot is not None and snapshot.matched_local_ip_device is not None:
            return account
    return None


def build_auto_switch_candidate(
    account_store: AccountStore,
    snapshots: dict[str, AccountTrafficSnapshot],
    recent_account_ids: list[str],
) -> PortalAccount | None:
    current_account = next(
        (account for account in account_store.accounts if account.id == account_store.selected_account_id),
        None,
    )
    if current_account is None:
        return None

    current_snapshot = snapshots.get(current_account.id)
    if current_snapshot is None or current_snapshot.progress_percent is None:
        return None
    if current_snapshot.progress_percent < 100:
        return None

    candidates: list[PortalAccount] = []
    for account in account_store.accounts:
        if account.id == current_account.id:
            continue
        snapshot = snapshots.get(account.id)
        if snapshot is None or snapshot.progress_percent is None:
            continue
        if snapshot.progress_percent >= 100:
            continue
        candidates.append(account)

    if not candidates:
        return None

    recent_rank = {account_id: idx for idx, account_id in enumerate(recent_account_ids)}
    return sorted(
        candidates,
        key=lambda account: (
            recent_rank.get(account.id, 10_000),
            account_store.accounts.index(account),
        ),
    )[0]


def build_status_card_order(
    account_store: AccountStore,
    snapshots: dict[str, AccountTrafficSnapshot],
    current_online_account_id: str,
    order_snapshot: list[str],
) -> list[str]:
    order_index = {account_id: index for index, account_id in enumerate(order_snapshot)}
    other_accounts = [
        account for account in account_store.accounts if account.id != current_online_account_id
    ]

    def account_sort_key(account: PortalAccount) -> tuple[bool, float, int]:
        snapshot = snapshots.get(account.id)
        progress_percent = snapshot.progress_percent if snapshot is not None else None
        missing_progress = progress_percent is None
        percent_for_sort = progress_percent if progress_percent is not None else -1.0
        return (
            missing_progress,
            -percent_for_sort,
            order_index.get(account.id, 10**9),
        )

    other_accounts.sort(
        key=account_sort_key,
    )
    final_order: list[str] = []
    if current_online_account_id:
        final_order.append(current_online_account_id)
    final_order.extend(account.id for account in other_accounts)
    return final_order
