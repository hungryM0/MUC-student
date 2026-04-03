from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime


@dataclass(slots=True)
class PortalHiddenFields:
    ac_id: str
    user_ip: str
    nas_ip: str
    user_mac: str


@dataclass(slots=True)
class LoginResult:
    success: bool
    message: str
    login_url: str
    hidden_fields: PortalHiddenFields
    response_text: str
    checked_at: datetime
    already_online: bool = False


@dataclass(slots=True)
class OnlineDeviceRecord:
    ip: str
    device_id: str
    logout_path: str


@dataclass(slots=True)
class AccountTrafficSnapshot:
    account_id: str
    used_traffic_text: str
    product_balance_text: str
    included_package_text: str
    online_device_count_text: str
    package_text: str
    status_text: str
    detail_text: str
    queried_at: datetime
    online_devices: list[OnlineDeviceRecord] = field(default_factory=list)
    matched_local_ip_device: OnlineDeviceRecord | None = None
    progress_percent: float | None = None
