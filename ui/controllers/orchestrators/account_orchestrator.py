from __future__ import annotations

from typing import TYPE_CHECKING

from domain.models.account import PortalAccount
from domain.policies.account_selection import build_auto_switch_candidate

if TYPE_CHECKING:
    from ui.controllers.main_window_controller import MainWindowController


class AccountOrchestrator:
    def __init__(self, controller: MainWindowController) -> None:
        self._controller = controller

    def get_account(self, account_id: str) -> PortalAccount | None:
        return self._controller._account_repo.get_account_by_id(self._controller.account_store, account_id)

    def select_account(self, account_id: str) -> None:
        controller = self._controller
        try:
            selected_account = controller._select_account_use_case.execute(account_id)
        except ValueError as exc:
            controller.warning_requested.emit("选择账号失败", str(exc))
            return

        controller.account_store = controller._account_repo.load_store()
        controller._recent_account_ids = controller._app_state_repo.mark_account_used(selected_account.id)
        self.reload_accounts()
        controller._log_service.log("INFO", f"登录目标账号已切换为：{selected_account.display_name}")

    def add_account(self, remark_name: str, username: str, password: str) -> None:
        controller = self._controller
        try:
            account = controller._add_account_use_case.execute(remark_name, username, password)
        except ValueError as exc:
            controller.warning_requested.emit("添加账号失败", str(exc))
            return
        self.reload_accounts()
        controller._log_service.log("SUCCESS", f"已添加账号：{account.display_name}")

    def edit_account(self, account_id: str, remark_name: str, username: str, password: str) -> None:
        controller = self._controller
        try:
            account = controller._edit_account_use_case.execute(account_id, remark_name, username, password)
        except ValueError as exc:
            controller.warning_requested.emit("编辑账号失败", str(exc))
            return
        self.reload_accounts()
        controller._log_service.log("SUCCESS", f"已更新账号：{account.display_name}")

    def delete_account(self, account_id: str) -> None:
        controller = self._controller
        account = controller._account_repo.get_account_by_id(controller.account_store, account_id)
        if account is None:
            controller.warning_requested.emit("删除账号失败", "找不到这个账号，可能已经被删了")
            return
        try:
            controller._delete_account_use_case.execute(account_id)
        except ValueError as exc:
            controller.warning_requested.emit("删除账号失败", str(exc))
            return

        controller._traffic_snapshots.pop(account_id, None)
        if controller._current_online_account_id == account_id:
            controller._current_online_account_id = ""
        self.reload_accounts()
        controller._log_service.log("INFO", f"已删除账号：{account.display_name}")

    def set_minimize_to_tray_on_close(self, enabled: bool) -> None:
        controller = self._controller
        controller.preferences.minimize_to_tray_on_close = bool(enabled)
        controller._app_state_repo.set_minimize_to_tray_on_close(controller.preferences.minimize_to_tray_on_close)
        mode_text = "最小化到托盘" if controller.preferences.minimize_to_tray_on_close else "直接退出程序"
        controller._log_service.log("INFO", f"关闭窗口行为已设置为：{mode_text}")
        controller._presentation.emit_all_views()

    def set_auto_switch_account_on_traffic_exhausted(self, enabled: bool) -> None:
        controller = self._controller
        controller.preferences.auto_switch_account_on_traffic_exhausted = bool(enabled)
        controller._app_state_repo.set_auto_switch_on_traffic_exhausted(
            controller.preferences.auto_switch_account_on_traffic_exhausted
        )
        if controller.preferences.auto_switch_account_on_traffic_exhausted:
            controller._log_service.log("INFO", "已开启：流量用完后自动切换账号")
        else:
            controller._log_service.log("INFO", "已关闭：流量用完后自动切换账号")
        controller._presentation.emit_all_views()

    def reload_accounts(self) -> None:
        controller = self._controller
        controller.account_store = controller._account_repo.load_store()
        valid_account_ids = {account.id for account in controller.account_store.accounts}
        controller._recent_account_ids = controller._app_state_repo.prune_recent_account_ids(valid_account_ids)
        controller._traffic_snapshots = {
            account_id: snapshot
            for account_id, snapshot in controller._traffic_snapshots.items()
            if account_id in valid_account_ids
        }
        if controller._current_online_account_id not in valid_account_ids:
            persisted_online_id = controller.account_store.current_online_account_id
            controller._current_online_account_id = persisted_online_id if persisted_online_id in valid_account_ids else ""
        controller._refresh.refresh_snapshot_local_matches()
        controller._presentation.save_cached_snapshots()
        controller._presentation.emit_all_views()

    def try_auto_switch_account_when_traffic_exhausted(self) -> None:
        controller = self._controller
        if not controller.preferences.auto_switch_account_on_traffic_exhausted:
            return
        target_account = build_auto_switch_candidate(
            controller.account_store,
            controller._traffic_snapshots,
            controller._recent_account_ids,
        )
        if target_account is None:
            current_account = controller._account_repo.get_selected_account(controller.account_store)
            current_snapshot = controller._traffic_snapshots.get(current_account.id) if current_account is not None else None
            if (
                current_account is not None
                and current_snapshot is not None
                and current_snapshot.progress_percent is not None
                and current_snapshot.progress_percent >= 100
            ):
                controller._log_service.log("INFO", f"{current_account.display_name} 流量已用完，但没有可自动切换的账号")
            return
        current_account = controller._account_repo.get_selected_account(controller.account_store)
        if current_account is None or target_account.id == current_account.id:
            return
        self.select_account(target_account.id)
        controller._log_service.log("INFO", f"检测到 {current_account.display_name} 流量已用完，已自动切换到：{target_account.display_name}")
