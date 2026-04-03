from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime


@dataclass(slots=True)
class AppState:
    last_login_time: datetime | None = None
    last_quota_refresh_time: datetime | None = None
    last_login_result: str = "未执行"
    last_login_message: str = "-"
