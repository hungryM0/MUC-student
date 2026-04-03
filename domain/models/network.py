from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime


@dataclass(slots=True)
class NetworkStatus:
    is_online: bool
    status_text: str
    ip: str
    checked_at: datetime
