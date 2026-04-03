from __future__ import annotations

from datetime import datetime

from domain.models.account import PortalAccount
from domain.models.traffic import AccountTrafficSnapshot
from infrastructure.network.self_service_panel_client import SelfServicePanelClient
from infrastructure.parsers.panel_home_parser import match_local_ip_device, parse_home_table, build_product_balance_texts
from infrastructure.parsers.online_device_parser import parse_online_devices
from domain.policies.traffic_math import parse_traffic_text_to_mb


class AccountTrafficService:
    def __init__(self, panel_client: SelfServicePanelClient) -> None:
        self._panel_client = panel_client

    def fetch_balances(
        self,
        accounts: list[PortalAccount],
        local_ip: str | None = None,
    ) -> list[AccountTrafficSnapshot]:
        snapshots: list[AccountTrafficSnapshot] = []
        for account in accounts:
            try:
                snapshots.append(self.fetch_balance(account, local_ip=local_ip))
            except Exception as exc:
                snapshots.append(
                    AccountTrafficSnapshot(
                        account_id=account.id,
                        used_traffic_text="-",
                        product_balance_text="-",
                        included_package_text="",
                        online_device_count_text="-",
                        package_text="-",
                        status_text="查询失败",
                        detail_text=str(exc),
                        queried_at=datetime.now(),
                        progress_percent=None,
                    )
                )
        return snapshots

    def detect_current_online_account_id(
        self,
        accounts: list[PortalAccount],
        local_ip: str | None,
        preferred_account_id: str = "",
    ) -> str:
        local_ip_text = (local_ip or "").strip()
        if not local_ip_text or local_ip_text == "unknown":
            return ""

        ordered_accounts = self._build_probe_order(accounts, preferred_account_id)
        for account in ordered_accounts:
            try:
                home_html = self._panel_client.fetch_authenticated_html(account, "/home")
            except Exception:
                continue
            online_devices = parse_online_devices(home_html)
            if match_local_ip_device(online_devices, local_ip_text) is not None:
                return account.id
        return ""

    def fetch_balance(self, account: PortalAccount, local_ip: str | None = None) -> AccountTrafficSnapshot:
        home_html = self._panel_client.fetch_authenticated_html(account, "/home")
        package_name, billing_policy, used_traffic, _ = parse_home_table(home_html)
        product_balance, included_package_text = build_product_balance_texts(home_html)
        online_devices = parse_online_devices(home_html)
        matched_local = match_local_ip_device(online_devices, local_ip)
        return AccountTrafficSnapshot(
            account_id=account.id,
            used_traffic_text=used_traffic,
            product_balance_text=product_balance,
            included_package_text=included_package_text,
            online_device_count_text=str(len(online_devices)),
            package_text=package_name,
            status_text="已同步",
            detail_text=f"计费策略：{billing_policy}",
            queried_at=datetime.now(),
            online_devices=online_devices,
            matched_local_ip_device=matched_local,
            progress_percent=self._build_progress_percent(used_traffic, billing_policy),
        )

    @staticmethod
    def _build_probe_order(accounts: list[PortalAccount], preferred_account_id: str) -> list[PortalAccount]:
        preferred_id = (preferred_account_id or "").strip()
        if not preferred_id:
            return list(accounts)
        preferred_accounts = [account for account in accounts if account.id == preferred_id]
        other_accounts = [account for account in accounts if account.id != preferred_id]
        return [*preferred_accounts, *other_accounts]

    @staticmethod
    def _build_progress_percent(used_traffic: str, billing_policy: str) -> float | None:
        used_mb = parse_traffic_text_to_mb(used_traffic)
        total_mb = parse_traffic_text_to_mb(billing_policy)
        if used_mb is None or total_mb is None or total_mb <= 0:
            return None
        return round(max(0.0, min(100.0, (used_mb / total_mb) * 100)), 1)
