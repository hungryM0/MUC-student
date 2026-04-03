from __future__ import annotations

from dataclasses import dataclass


@dataclass(slots=True, frozen=True)
class AppSettings:
    portal_url: str = "http://rz.muc.edu.cn/srun_portal_pc.php?ac_id=1&"
    traffic_portal_url: str = "http://192.168.2.231:8800/home"
    preferred_interface_name: str = "WLAN"
    preferred_source_ip: str = ""
    bind_preferred_source_ip: bool = False
    navigation_expand_width: int = 170

    @classmethod
    def load(cls) -> "AppSettings":
        return cls()
