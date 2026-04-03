from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(slots=True)
class UserPreferences:
    minimize_to_tray_on_close: bool = False
    auto_switch_account_on_traffic_exhausted: bool = False
    recent_account_ids: list[str] = field(default_factory=list)
