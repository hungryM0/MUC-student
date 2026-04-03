from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime


@dataclass(slots=True)
class PortalAccount:
    id: str
    remark_name: str
    username: str
    password: str

    @property
    def display_name(self) -> str:
        return f"{self.remark_name}（{self.username}）"


@dataclass(slots=True)
class CachedTrafficSnapshot:
    used_traffic_text: str
    product_balance_text: str
    included_package_text: str
    online_device_count_text: str
    package_text: str
    status_text: str
    detail_text: str
    queried_at: datetime | None = None
    progress_percent: float | None = None


@dataclass(slots=True)
class AccountStore:
    selected_account_id: str = ""
    accounts: list[PortalAccount] = field(default_factory=list)
    current_online_account_id: str = ""
    status_card_order_snapshot: list[str] = field(default_factory=list)
    cached_traffic_snapshots: dict[str, CachedTrafficSnapshot] = field(default_factory=dict)
