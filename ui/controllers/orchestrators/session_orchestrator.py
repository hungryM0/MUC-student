from __future__ import annotations

from datetime import datetime
from typing import TYPE_CHECKING

from application.use_cases.login_selected_account import LoginWorkflowResult
from application.use_cases.logout_local_device import LogoutWorkflowResult

if TYPE_CHECKING:
    from ui.controllers.main_window_controller import MainWindowController


class SessionOrchestrator:
    def __init__(self, controller: MainWindowController) -> None:
        self._controller = controller

    def start_login(self) -> None:
        controller = self._controller
        if controller._closing:
            return
        if controller._login_running:
            controller._log_service.log("INFO", "HTTP 登录正在执行中，请稍等")
            return

        selected_account = controller._account_repo.get_selected_account(controller.account_store)
        if selected_account is None:
            controller._log_service.log("ERROR", "当前没有可用账号，请先去状态页添加账号")
            controller._presentation.emit_all_views()
            return

        controller._login_running = True
        controller._log_service.log(
            "INFO",
            f"开始 HTTP 认证，账号={selected_account.display_name}, portal={controller.settings.portal_url}",
        )
        controller._presentation.emit_all_views()
        controller._runner.run(
            lambda: controller._login_use_case.execute(selected_account, list(controller.account_store.accounts)),
            self.on_login_success,
            self.on_login_failure,
        )

    def logout_local_device_for_account(self, account_id: str) -> None:
        controller = self._controller
        if controller._closing:
            return
        if controller._local_logout_running or controller._login_running:
            controller._log_service.log("INFO", "当前有任务执行中，请稍后再试本机下线")
            return
        if account_id != controller._current_online_account_id:
            controller.warning_requested.emit("下线失败", "这个账号不是当前登录账号，不能执行本机下线")
            return

        snapshot = controller._traffic_snapshots.get(account_id)
        if snapshot is None or snapshot.matched_local_ip_device is None:
            controller.warning_requested.emit("下线失败", "当前账号在线列表里没有识别到本机设备")
            return

        account = controller._account_repo.get_account_by_id(controller.account_store, account_id)
        if account is None:
            controller.warning_requested.emit("下线失败", "找不到这个账号，可能已经被删了")
            return

        controller._local_logout_running = True
        controller._presentation.emit_all_views()
        controller._runner.run(
            lambda: controller._logout_local_device_use_case.execute(account),
            self.on_logout_local_success,
            self.on_logout_local_failure,
        )

    def on_login_success(self, result: object) -> None:
        controller = self._controller
        controller._login_running = False
        if not isinstance(result, LoginWorkflowResult):
            self.on_login_failure("登录返回值类型错误")
            return

        if result.detected_local_ip:
            controller._current_ip = result.detected_local_ip
        if result.prelogout_performed:
            controller._log_service.log("INFO", f"切号前已先下线当前账号本机设备：{result.prelogout_account_name}")
        elif result.prelogout_note:
            controller._log_service.log("INFO", result.prelogout_note)

        controller.app_state.last_login_time = result.login_result.checked_at
        controller.app_state.last_login_result = "成功" if result.login_result.success else "失败"
        controller.app_state.last_login_message = result.login_result.message
        controller._presentation.save_app_state()
        controller._log_service.log("SUCCESS" if result.login_result.success else "ERROR", result.login_result.message)
        controller._log_service.log(
            "INFO",
            "HTTP 登录参数："
            f"ac_id={result.login_result.hidden_fields.ac_id}, "
            f"user_ip={result.login_result.hidden_fields.user_ip}, "
            f"nas_ip={result.login_result.hidden_fields.nas_ip}, "
            f"user_mac={result.login_result.hidden_fields.user_mac}",
        )
        controller._log_service.log("INFO", f"接口原始返回：{result.login_result.response_text or 'empty'}")
        if result.login_result.success:
            controller._recent_account_ids = controller._preferences_repo.mark_account_used(
                controller.account_store.selected_account_id
            )
        controller._presentation.emit_all_views()

    def on_login_failure(self, error_text: str) -> None:
        controller = self._controller
        controller._login_running = False
        controller.app_state.last_login_time = datetime.now()
        controller.app_state.last_login_result = "失败"
        controller.app_state.last_login_message = error_text
        controller._presentation.save_app_state()
        controller._log_service.log("ERROR", "登录任务异常", error=error_text)
        if "下线" in error_text and "本机" in error_text:
            controller.warning_requested.emit("切号失败", f"当前账号本机下线失败：{error_text}")
        controller._presentation.emit_all_views()

    def on_logout_local_success(self, result: object) -> None:
        controller = self._controller
        try:
            if not isinstance(result, LogoutWorkflowResult):
                self.on_logout_local_failure("本机下线返回值类型错误")
                return
            controller._current_ip = result.detected_local_ip
            controller._log_service.log("SUCCESS", result.message)
            controller._refresh.refresh_status_page_data(force_quota_refresh=True)
        finally:
            controller._local_logout_running = False
            controller._presentation.emit_all_views()

    def on_logout_local_failure(self, error_text: str) -> None:
        controller = self._controller
        try:
            controller._log_service.log("ERROR", "本机下线任务异常", error=error_text)
            controller.warning_requested.emit("本机下线失败", error_text)
        finally:
            controller._local_logout_running = False
            controller._presentation.emit_all_views()
