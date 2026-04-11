from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime

from application.view_models import (
    AccountCardViewModel,
    AccountOptionViewModel,
    HomePageViewModel,
    LoginSummaryViewModel,
    PoolQuotaViewModel,
    QuotaCardViewModel,
    SettingsViewModel,
    StatusPageViewModel,
)
from domain.models.account import AccountStore, CachedTrafficSnapshot
from domain.models.app_state import AppState
from domain.models.preferences import UserPreferences
from domain.models.traffic import AccountTrafficSnapshot
from domain.policies.account_selection import build_status_card_order
from domain.policies.traffic_math import (
    build_pool_quota_summary,
    build_remaining_traffic_text,
    parse_traffic_text_to_mb,
)
from infrastructure.settings import AppSettings


@dataclass(slots=True)
class PresentationState:
    account_store: AccountStore
    app_state: AppState
    preferences: UserPreferences
    snapshots: dict[str, AccountTrafficSnapshot]
    current_ip: str
    current_online_account_id: str
    status_card_order_snapshot: list[str]
    login_running: bool = False
    status_refresh_running: bool = False
    traffic_refresh_running: bool = False
    local_logout_running: bool = False


class DashboardViewMapper:
    def build_home_page(self, state: PresentationState) -> HomePageViewModel:
        account_by_id = {account.id: account for account in state.account_store.accounts}
        home_account_order = self._build_home_account_order(state)
        quota_account_id = state.current_online_account_id.strip()
        if quota_account_id not in account_by_id:
            quota_account_id = state.account_store.selected_account_id
        effective_selected_account_id = state.account_store.selected_account_id
        if effective_selected_account_id not in home_account_order:
            effective_selected_account_id = home_account_order[0] if home_account_order else ""
        selected_account = account_by_id.get(quota_account_id)
        snapshot = state.snapshots.get(quota_account_id)
        loading = snapshot is None or snapshot.status_text == "查询中..."
        quota_card = QuotaCardViewModel(
            remark_name=selected_account.remark_name if selected_account else None,
            used_traffic_text=snapshot.used_traffic_text if snapshot else "-",
            product_balance_text=snapshot.product_balance_text if snapshot else "-",
            online_device_count_text=snapshot.online_device_count_text if snapshot else "-",
            included_package_text=snapshot.included_package_text if snapshot else "",
            progress_percent=snapshot.progress_percent if snapshot else None,
            loading=loading if selected_account is not None else False,
        )
        message = state.app_state.last_login_message
        if not state.account_store.accounts and state.app_state.last_login_result == "未执行":
            message = "请先去状态页添加账号"
        return HomePageViewModel(
            ip=state.current_ip,
            login_time_text=self._format_time(state.app_state.last_login_time),
            accounts=[
                AccountOptionViewModel(
                    account_id=account.id,
                    label=f"{account.remark_name}（{self._build_available_text(state.snapshots.get(account.id))} 可用）",
                )
                for account_id in home_account_order
                if (account := account_by_id.get(account_id)) is not None
            ],
            selected_account_id=effective_selected_account_id,
            login_button_mode="running" if state.login_running else "start",
            login_summary=LoginSummaryViewModel(
                result_text=state.app_state.last_login_result,
                login_time_text=self._format_time(state.app_state.last_login_time),
                message=message,
            ),
            quota_card=quota_card,
        )

    def home_accounts_need_background_refresh(self, state: PresentationState) -> bool:
        if state.traffic_refresh_running:
            return False
        return any(
            self._snapshot_needs_refresh(state.snapshots.get(account_id))
            for account_id in self._build_home_account_order(state)
        )

    def build_status_page(self, state: PresentationState) -> StatusPageViewModel:
        used_text, total_text, included_text, progress_percent = build_pool_quota_summary(
            state.account_store,
            state.snapshots,
        )
        order = build_status_card_order(
            state.account_store,
            state.snapshots,
            state.current_online_account_id,
            state.status_card_order_snapshot,
        )
        cards: list[AccountCardViewModel] = []
        for account_id in order:
            account = next((item for item in state.account_store.accounts if item.id == account_id), None)
            if account is None:
                continue
            snapshot = state.snapshots.get(account.id)
            is_current_online = account.id == state.current_online_account_id
            can_logout_local = (
                is_current_online and snapshot is not None and snapshot.matched_local_ip_device is not None
            )
            cards.append(
                AccountCardViewModel(
                    account_id=account.id,
                    remark_name=account.remark_name,
                    username=account.username,
                    used_traffic_text=snapshot.used_traffic_text if snapshot else "-",
                    product_balance_text=snapshot.product_balance_text if snapshot else "-",
                    included_package_text=snapshot.included_package_text if snapshot else "",
                    online_device_count_text=snapshot.online_device_count_text if snapshot else "-",
                    package_text=snapshot.package_text if snapshot else "-",
                    status_text=snapshot.status_text if snapshot else "未查询",
                    detail_text=(snapshot.detail_text if snapshot else "会按刷新策略自动更新，或手动点击右上角“刷新状态”"),
                    updated_at_text=self._format_time(snapshot.queried_at) if snapshot else "-",
                    progress_percent=snapshot.progress_percent if snapshot else None,
                    is_current_online_account=is_current_online,
                    can_logout_local_device=can_logout_local,
                    logout_action_enabled=(can_logout_local and not state.local_logout_running and not state.login_running),
                )
            )
        return StatusPageViewModel(
            pool_quota=PoolQuotaViewModel(
                used_traffic_text=used_text,
                product_balance_text=total_text,
                included_package_text=included_text,
                progress_percent=progress_percent,
                loading=state.traffic_refresh_running,
            ),
            cards=cards,
            refreshing=state.status_refresh_running or state.traffic_refresh_running or state.local_logout_running,
        )

    @staticmethod
    def build_settings_view(settings: AppSettings, preferences: UserPreferences) -> SettingsViewModel:
        return SettingsViewModel(
            portal_url=settings.portal_url,
            traffic_portal_url=settings.traffic_portal_url,
            minimize_to_tray_on_close=preferences.minimize_to_tray_on_close,
            auto_switch_account_on_traffic_exhausted=preferences.auto_switch_account_on_traffic_exhausted,
        )

    @staticmethod
    def restore_cached_snapshots(cached_snapshots: dict[str, CachedTrafficSnapshot]) -> dict[str, AccountTrafficSnapshot]:
        restored: dict[str, AccountTrafficSnapshot] = {}
        for account_id, cached_snapshot in cached_snapshots.items():
            restored[account_id] = AccountTrafficSnapshot(
                account_id=account_id,
                used_traffic_text=cached_snapshot.used_traffic_text,
                product_balance_text=cached_snapshot.product_balance_text,
                included_package_text=cached_snapshot.included_package_text,
                online_device_count_text=cached_snapshot.online_device_count_text,
                package_text=cached_snapshot.package_text,
                status_text=cached_snapshot.status_text,
                detail_text=cached_snapshot.detail_text,
                queried_at=cached_snapshot.queried_at or datetime.now(),
                online_devices=[],
                matched_local_ip_device=None,
                progress_percent=cached_snapshot.progress_percent,
            )
        return restored

    @staticmethod
    def to_cached_snapshots(snapshots: dict[str, AccountTrafficSnapshot]) -> dict[str, CachedTrafficSnapshot]:
        cached: dict[str, CachedTrafficSnapshot] = {}
        for account_id, snapshot in snapshots.items():
            if snapshot.status_text in {"查询中...", "查询失败"}:
                continue
            cached[account_id] = CachedTrafficSnapshot(
                used_traffic_text=snapshot.used_traffic_text,
                product_balance_text=snapshot.product_balance_text,
                included_package_text=snapshot.included_package_text,
                online_device_count_text=snapshot.online_device_count_text,
                package_text=snapshot.package_text,
                status_text=snapshot.status_text,
                detail_text=snapshot.detail_text,
                queried_at=snapshot.queried_at,
                progress_percent=snapshot.progress_percent,
            )
        return cached

    @staticmethod
    def _format_time(dt: datetime | None) -> str:
        if dt is None:
            return "-"
        return dt.strftime("%Y-%m-%d %H:%M:%S")

    @staticmethod
    def _build_available_text(snapshot: AccountTrafficSnapshot | None) -> str:
        if snapshot is None:
            return "待刷新"
        remaining_text = build_remaining_traffic_text(
            snapshot.product_balance_text,
            snapshot.used_traffic_text,
        )
        if remaining_text is None:
            return "待刷新"
        return remaining_text

    @staticmethod
    def _snapshot_needs_refresh(snapshot: AccountTrafficSnapshot | None) -> bool:
        if snapshot is None:
            return True
        return build_remaining_traffic_text(snapshot.product_balance_text, snapshot.used_traffic_text) is None

    @staticmethod
    def _build_home_account_order(state: PresentationState) -> list[str]:
        status_order = build_status_card_order(
            state.account_store,
            state.snapshots,
            state.current_online_account_id,
            state.status_card_order_snapshot,
        )
        order_index = {account_id: index for index, account_id in enumerate(status_order)}
        candidate_ids = [
            account_id
            for account_id in status_order
            if account_id != state.current_online_account_id
            and DashboardViewMapper._snapshot_can_login(state.snapshots.get(account_id))
        ]
        candidate_ids.sort(
            key=lambda account_id: (
                DashboardViewMapper._remaining_sort_rank(state.snapshots.get(account_id)),
                order_index.get(account_id, 10**9),
            )
        )
        return candidate_ids

    @staticmethod
    def _snapshot_can_login(snapshot: AccountTrafficSnapshot | None) -> bool:
        if snapshot is None:
            return True
        remaining_text = build_remaining_traffic_text(snapshot.product_balance_text, snapshot.used_traffic_text)
        if remaining_text is None:
            return True
        remaining_mb = parse_traffic_text_to_mb(remaining_text)
        return remaining_mb is None or remaining_mb > 0

    @staticmethod
    def _remaining_sort_rank(snapshot: AccountTrafficSnapshot | None) -> tuple[bool, float]:
        if snapshot is None:
            return (True, 0.0)
        remaining_text = build_remaining_traffic_text(snapshot.product_balance_text, snapshot.used_traffic_text)
        if remaining_text is None:
            return (True, 0.0)
        remaining_mb = parse_traffic_text_to_mb(remaining_text)
        if remaining_mb is None:
            return (True, 0.0)
        return (False, -remaining_mb)

