from __future__ import annotations

from datetime import datetime
import ipaddress
import socket

from domain.models.network import NetworkStatus
from infrastructure.settings import AppSettings


class NetworkStatusService:
    def __init__(self, settings: AppSettings) -> None:
        self._settings = settings

    def detect_network_status(self) -> NetworkStatus:
        ip = self._fetch_private_ipv4()
        is_online = ip not in ("unknown", "")
        return NetworkStatus(
            is_online=is_online,
            status_text="在线" if is_online else "未认证",
            ip=ip if ip else "unknown",
            checked_at=datetime.now(),
        )

    def _fetch_private_ipv4(self) -> str:
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
                sock.connect(("8.8.8.8", 80))
                ip = str(sock.getsockname()[0]).strip()
                if self._is_private_ipv4(ip):
                    return ip
        except Exception:
            pass

        try:
            _, _, candidates = socket.gethostbyname_ex(socket.gethostname())
            for candidate in candidates:
                ip = str(candidate).strip()
                if self._is_private_ipv4(ip):
                    return ip
        except Exception:
            pass

        configured_ip = str(self._settings.preferred_source_ip).strip()
        if self._settings.bind_preferred_source_ip and self._is_private_ipv4(configured_ip):
            return configured_ip
        return "unknown"

    @staticmethod
    def _is_private_ipv4(ip_text: str) -> bool:
        try:
            ip_obj = ipaddress.ip_address(ip_text)
            return ip_obj.version == 4 and ip_obj.is_private
        except ValueError:
            return False

