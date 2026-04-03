from __future__ import annotations

from collections import deque
from dataclasses import dataclass
from datetime import datetime
from typing import Callable


@dataclass(slots=True)
class LogEntry:
    timestamp: datetime
    status: str
    message: str
    error: str = ""

    def to_line(self) -> str:
        time_text = self.timestamp.strftime("%Y-%m-%d %H:%M:%S")
        err = f" | error={self.error}" if self.error else ""
        return f"[{time_text}] [{self.status}] {self.message}{err}"


class LogService:
    def __init__(self, max_entries: int = 500) -> None:
        self._entries: deque[LogEntry] = deque(maxlen=max_entries)
        self._listeners: list[Callable[[LogEntry], None]] = []

    def add_listener(self, listener: Callable[[LogEntry], None]) -> None:
        self._listeners.append(listener)

    def log(self, status: str, message: str, error: str = "") -> LogEntry:
        entry = LogEntry(timestamp=datetime.now(), status=status, message=message, error=error)
        self._entries.append(entry)
        print(entry.to_line())
        for listener in self._listeners:
            listener(entry)
        return entry

    def get_entries(self) -> list[LogEntry]:
        return list(self._entries)
