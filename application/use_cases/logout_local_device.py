from __future__ import annotations

from dataclasses import dataclass

from domain.models.account import PortalAccount
from infrastructure.network.network_status_service import NetworkStatusService
from infrastructure.network.self_service_panel_client import SelfServicePanelClient


@dataclass(slots=True)
class LogoutWorkflowResult:
    detected_local_ip: str
    message: str


class LogoutLocalDeviceUseCase:
    def __init__(
        self,
        network_status_service: NetworkStatusService,
        panel_client: SelfServicePanelClient,
    ) -> None:
        self._network_status_service = network_status_service
        self._panel_client = panel_client

    def execute(self, account: PortalAccount) -> LogoutWorkflowResult:
        status = self._network_status_service.detect_network_status()
        local_ip = status.ip if status.ip else "unknown"
        if local_ip in ("", "unknown"):
            raise RuntimeError("本机内网 IP 未识别到，无法执行本机下线")

        return LogoutWorkflowResult(
            detected_local_ip=local_ip,
            message=self._panel_client.logout_local_device(account, local_ip),
        )
