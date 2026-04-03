from __future__ import annotations

from application.services.account_traffic_service import AccountTrafficService
from infrastructure.network.network_status_service import NetworkStatusService


class RefreshNetworkStatusUseCase:
    def __init__(self, network_status_service: NetworkStatusService) -> None:
        self._network_status_service = network_status_service

    def execute(self):
        return self._network_status_service.detect_network_status()


class RefreshAccountSnapshotsUseCase:
    def __init__(self, traffic_service: AccountTrafficService) -> None:
        self._traffic_service = traffic_service

    def execute(self, accounts, local_ip: str | None = None):
        return self._traffic_service.fetch_balances(accounts=accounts, local_ip=local_ip)


class VerifyOnlineAccountUseCase:
    def __init__(self, traffic_service: AccountTrafficService) -> None:
        self._traffic_service = traffic_service

    def execute(self, accounts, local_ip: str | None, preferred_account_id: str = "") -> str:
        return self._traffic_service.detect_current_online_account_id(
            accounts=accounts,
            local_ip=local_ip,
            preferred_account_id=preferred_account_id,
        )
