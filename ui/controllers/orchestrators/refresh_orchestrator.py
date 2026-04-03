from __future__ import annotations

from datetime import datetime, timedelta
from typing import TYPE_CHECKING

from domain.models.traffic import AccountTrafficSnapshot

if TYPE_CHECKING:
    from ui.controllers.main_window_controller import MainWindowController


class RefreshOrchestrator:
    _AUTO_QUOTA_REFRESH_COOLDOWN = timedelta(minutes=30)

    def __init__(self, controller: MainWindowController) -> None:
        self._controller = controller

    def refresh_status_page_data(self, force_quota_refresh: bool = False) -> None:
        controller = self._controller
        self.refresh_network_status()
        quota_refresh_started = self.refresh_account_snapshots(force_refresh=force_quota_refresh)
        if not quota_refresh_started:
            controller._pending_online_account_verify = True
            self.try_start_online_account_verify()

    def refresh_network_status(self) -> None:
        controller = self._controller
        if controller._closing or controller._status_refresh_running:
            return
        controller._status_refresh_running = True
        controller._presentation.emit_all_views()
        controller._runner.run(controller._refresh_network_use_case.execute, self.on_status_success, self.on_status_failure)

    def refresh_account_snapshots(self, force_refresh: bool = False) -> bool:
        controller = self._controller
        if controller._closing or controller._traffic_refresh_running:
            return False
        accounts = list(controller.account_store.accounts)
        if not accounts:
            controller._traffic_snapshots.clear()
            controller._current_online_account_id = ""
            controller._presentation.emit_all_views()
            return False
        if not force_refresh and self.should_skip_auto_quota_refresh():
            return False

        controller.app_state.last_quota_refresh_time = datetime.now()
        controller._presentation.save_app_state()
        controller._traffic_refresh_running = True
        loading_time = controller.app_state.last_quota_refresh_time or datetime.now()
        for account in accounts:
            controller._traffic_snapshots[account.id] = AccountTrafficSnapshot(
                account_id=account.id,
                used_traffic_text="-",
                product_balance_text="-",
                included_package_text="",
                online_device_count_text="-",
                package_text="-",
                status_text="查询中...",
                detail_text="正在刷新这个账号的流量与套餐信息",
                queried_at=loading_time,
                progress_percent=None,
            )
        controller._presentation.emit_all_views()
        local_ip = controller._current_ip if controller._current_ip not in ("", "unknown") else None
        controller._runner.run(
            lambda: controller._refresh_snapshots_use_case.execute(accounts, local_ip=local_ip),
            self.on_balance_refresh_success,
            self.on_balance_refresh_failure,
        )
        return True

    def on_status_success(self, status: object) -> None:
        controller = self._controller
        try:
            if not hasattr(status, "ip"):
                controller._current_ip = "unknown"
                controller._log_service.log("ERROR", "状态检测异常", error="网络状态返回值类型错误")
            else:
                controller._current_ip = status.ip
            self.refresh_snapshot_local_matches()
            controller._presentation.emit_all_views()
        finally:
            controller._status_refresh_running = False
            controller._presentation.emit_all_views()
            self.try_start_online_account_verify()

    def on_status_failure(self, error_text: str) -> None:
        controller = self._controller
        try:
            controller._current_ip = "unknown"
            self.refresh_snapshot_local_matches()
            controller._log_service.log("ERROR", "状态检测异常", error=error_text)
            controller._presentation.emit_all_views()
        finally:
            controller._status_refresh_running = False
            controller._presentation.emit_all_views()
            self.try_start_online_account_verify()

    def on_balance_refresh_success(self, result: object) -> None:
        controller = self._controller
        try:
            if not isinstance(result, list) or any(not isinstance(item, AccountTrafficSnapshot) for item in result):
                controller._log_service.log("ERROR", "流量查询任务异常", error="流量查询返回值类型错误")
            else:
                valid_account_ids = {account.id for account in controller.account_store.accounts}
                for snapshot in result:
                    if snapshot.account_id in valid_account_ids:
                        controller._traffic_snapshots[snapshot.account_id] = snapshot
                self.refresh_snapshot_local_matches()
                controller._presentation.save_cached_snapshots()
                controller._account.try_auto_switch_account_when_traffic_exhausted()
            controller._presentation.emit_all_views()
        finally:
            controller._traffic_refresh_running = False
            controller._presentation.emit_all_views()

    def on_balance_refresh_failure(self, error_text: str) -> None:
        controller = self._controller
        try:
            controller._log_service.log("ERROR", "流量查询任务异常", error=error_text)
            fail_time = datetime.now()
            for account in controller.account_store.accounts:
                controller._traffic_snapshots[account.id] = AccountTrafficSnapshot(
                    account_id=account.id,
                    used_traffic_text="-",
                    product_balance_text="-",
                    included_package_text="",
                    online_device_count_text="-",
                    package_text="-",
                    status_text="查询失败",
                    detail_text=error_text,
                    queried_at=fail_time,
                    progress_percent=None,
                )
            controller._presentation.emit_all_views()
        finally:
            controller._traffic_refresh_running = False
            controller._presentation.emit_all_views()

    def try_start_online_account_verify(self) -> None:
        controller = self._controller
        if not controller._pending_online_account_verify or controller._closing:
            return
        if controller._online_account_verify_running or controller._traffic_refresh_running or controller._status_refresh_running:
            return

        local_ip = controller._current_ip.strip()
        if not local_ip or local_ip == "unknown":
            return
        accounts = list(controller.account_store.accounts)
        if not accounts:
            controller._pending_online_account_verify = False
            controller._current_online_account_id = ""
            controller._presentation.emit_all_views()
            return

        controller._online_account_verify_running = True
        controller._pending_online_account_verify = False
        preferred_account_id = controller._current_online_account_id
        controller._runner.run(
            lambda: controller._verify_online_account_use_case.execute(
                accounts=accounts,
                local_ip=local_ip,
                preferred_account_id=preferred_account_id,
            ),
            self.on_online_account_verify_success,
            self.on_online_account_verify_failure,
        )

    def on_online_account_verify_success(self, result: object) -> None:
        controller = self._controller
        try:
            if not isinstance(result, str):
                controller._log_service.log("ERROR", "在线账号校验异常", error="在线账号校验返回值类型错误")
                return
            verified_account_id = result.strip()
            valid_account_ids = {account.id for account in controller.account_store.accounts}
            if verified_account_id not in valid_account_ids:
                verified_account_id = ""
            controller._current_online_account_id = verified_account_id
            controller._presentation.refresh_status_order_snapshot()
            controller._presentation.emit_all_views()
            controller._presentation.save_cached_snapshots()
        finally:
            controller._online_account_verify_running = False

    def on_online_account_verify_failure(self, error_text: str) -> None:
        controller = self._controller
        try:
            controller._log_service.log("ERROR", "在线账号校验异常", error=error_text)
        finally:
            controller._online_account_verify_running = False

    def refresh_snapshot_local_matches(self) -> None:
        controller = self._controller
        local_ip = controller._current_ip if controller._current_ip not in ("", "unknown") else ""
        for snapshot in controller._traffic_snapshots.values():
            matched = None
            if local_ip:
                for device in snapshot.online_devices:
                    if device.ip.strip() == local_ip:
                        matched = device
                        break
            snapshot.matched_local_ip_device = matched
        self.rebuild_current_online_account_id()
        controller._presentation.refresh_status_order_snapshot()

    def rebuild_current_online_account_id(self) -> None:
        controller = self._controller
        for account in controller.account_store.accounts:
            snapshot = controller._traffic_snapshots.get(account.id)
            if snapshot is not None and snapshot.matched_local_ip_device is not None:
                controller._current_online_account_id = account.id
                return
        valid_account_ids = {account.id for account in controller.account_store.accounts}
        if controller._current_online_account_id in valid_account_ids:
            return
        controller._current_online_account_id = ""

    def should_skip_auto_quota_refresh(self) -> bool:
        now = datetime.now()
        controller = self._controller
        return self._is_within_auto_quota_refresh_cooldown(
            controller.app_state.last_quota_refresh_time,
            now,
        ) or self._is_within_auto_quota_refresh_cooldown(controller.app_state.last_login_time, now)

    def _is_within_auto_quota_refresh_cooldown(self, target_time: datetime | None, now: datetime) -> bool:
        if target_time is None:
            return False
        delta = now - target_time
        if delta.total_seconds() < 0:
            return True
        return delta <= self._AUTO_QUOTA_REFRESH_COOLDOWN
