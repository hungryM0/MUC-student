from __future__ import annotations

from typing import TYPE_CHECKING

from application.dashboard_view_mapper import PresentationState
from application.services.log_service import LogEntry
from domain.policies.account_selection import build_status_card_order

if TYPE_CHECKING:
    from ui.controllers.main_window_controller import MainWindowController


class PresentationOrchestrator:
    def __init__(self, controller: MainWindowController) -> None:
        self._controller = controller

    def get_existing_log_lines(self) -> list[str]:
        return [entry.to_line() for entry in self._controller._log_service.get_entries()]

    def forward_log_to_ui(self, entry: LogEntry) -> None:
        self._controller.log_arrived.emit(entry.to_line())

    def refresh_status_order_snapshot(self) -> None:
        controller = self._controller
        controller._status_card_order_snapshot = build_status_card_order(
            controller.account_store,
            controller._traffic_snapshots,
            controller._current_online_account_id,
            controller._status_card_order_snapshot,
        )

    def save_app_state(self) -> None:
        try:
            self._controller._app_state_repo.save_state(self._controller.app_state)
        except OSError as exc:
            self._controller._log_service.log("ERROR", "保存最近登录状态失败", error=str(exc))

    def save_cached_snapshots(self) -> None:
        controller = self._controller
        try:
            controller._account_repo.save_cached_traffic_snapshots(
                cached_snapshots=controller._view_mapper.to_cached_snapshots(controller._traffic_snapshots),
                current_online_account_id=controller._current_online_account_id,
                status_card_order_snapshot=controller._status_card_order_snapshot,
            )
        except (OSError, RuntimeError) as exc:
            controller._log_service.log("ERROR", "保存本地配额缓存失败", error=str(exc))

    def emit_all_views(self) -> None:
        controller = self._controller
        state = PresentationState(
            account_store=controller.account_store,
            app_state=controller.app_state,
            preferences=controller.preferences,
            snapshots=controller._traffic_snapshots,
            current_ip=controller._current_ip,
            current_online_account_id=controller._current_online_account_id,
            status_card_order_snapshot=controller._status_card_order_snapshot,
            login_running=controller._login_running,
            status_refresh_running=controller._status_refresh_running,
            traffic_refresh_running=controller._traffic_refresh_running,
            local_logout_running=controller._local_logout_running,
        )
        home_view = controller._view_mapper.build_home_page(state)
        status_view = controller._view_mapper.build_status_page(state)
        controller.home_changed.emit(home_view)
        controller.status_changed.emit(status_view)
        controller.settings_changed.emit(controller._view_mapper.build_settings_view(controller.settings, controller.preferences))
        if controller._view_mapper.home_accounts_need_background_refresh(state):
            controller.refresh_account_snapshots(force_refresh=True)
