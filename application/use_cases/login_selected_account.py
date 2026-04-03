from __future__ import annotations

from dataclasses import dataclass

from application.services.account_traffic_service import AccountTrafficService
from domain.models.account import PortalAccount
from domain.models.traffic import LoginResult
from domain.policies.account_selection import find_current_online_account
from infrastructure.network.auth_portal_client import AuthPortalClient
from infrastructure.network.network_status_service import NetworkStatusService
from infrastructure.network.self_service_panel_client import SelfServicePanelClient


@dataclass(slots=True)
class LoginWorkflowResult:
    login_result: LoginResult
    detected_local_ip: str
    prelogout_performed: bool
    prelogout_account_name: str
    prelogout_note: str


class LoginSelectedAccountUseCase:
    def __init__(
        self,
        auth_client: AuthPortalClient,
        panel_client: SelfServicePanelClient,
        network_status_service: NetworkStatusService,
        traffic_service: AccountTrafficService,
    ) -> None:
        self._auth_client = auth_client
        self._panel_client = panel_client
        self._network_status_service = network_status_service
        self._traffic_service = traffic_service

    def execute(self, target_account: PortalAccount, accounts_for_check: list[PortalAccount]) -> LoginWorkflowResult:
        detected_status = self._network_status_service.detect_network_status()
        local_ip = detected_status.ip if detected_status.ip else "unknown"

        current_online_account = None
        prelogout_note = ""
        if local_ip not in ("", "unknown") and accounts_for_check:
            snapshots = self._traffic_service.fetch_balances(accounts_for_check, local_ip=local_ip)
            current_online_account = find_current_online_account(accounts_for_check, snapshots)
            if current_online_account is None:
                prelogout_note = f"没在任何账号在线列表里找到本机 IP（{local_ip}），跳过预下线"
        else:
            prelogout_note = "未识别到本机内网 IP，跳过预下线"

        prelogout_performed = False
        prelogout_account_name = ""
        if current_online_account is not None and current_online_account.id != target_account.id:
            self._panel_client.logout_local_device(current_online_account, local_ip)
            prelogout_performed = True
            prelogout_account_name = current_online_account.display_name
            prelogout_note = ""

        login_result = self._auth_client.verify_login(target_account)
        if login_result.already_online:
            if current_online_account is not None and current_online_account.id == target_account.id:
                login_result.success = True
                login_result.message = f"当前 IP 已在线（{target_account.display_name}），无需重复登录"
            elif prelogout_performed and prelogout_account_name:
                login_result.success = False
                login_result.message = (
                    f"当前 IP 仍显示已在线，且不是目标账号；预下线账号={prelogout_account_name}，"
                    "疑似下线未生效，请重试本机下线后再登录"
                )
            else:
                login_result.success = False
                login_result.message = "当前 IP 已在线，但无法确认是不是目标账号，请先本机下线后再登录"
        return LoginWorkflowResult(
            login_result=login_result,
            detected_local_ip=local_ip,
            prelogout_performed=prelogout_performed,
            prelogout_account_name=prelogout_account_name,
            prelogout_note=prelogout_note,
        )
