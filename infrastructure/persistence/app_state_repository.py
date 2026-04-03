from __future__ import annotations

import json
from datetime import datetime

from domain.models.app_state import AppState
from infrastructure.persistence.file_write_utils import write_json_atomic
from infrastructure.persistence.runtime_paths_provider import RuntimePathsProvider


class AppStateRepository:
    def __init__(self, paths: RuntimePathsProvider) -> None:
        self._paths = paths
        self._state_path = paths.get_app_state_path()

    def load_state(self) -> AppState:
        if self._state_path.exists():
            return self._load_from_path(self._state_path)

        legacy_path = self._paths.get_legacy_app_state_path()
        if legacy_path.exists():
            return self._load_from_path(legacy_path)
        return AppState()

    def save_state(self, state: AppState) -> None:
        payload = {
            "last_login_time": state.last_login_time.isoformat() if state.last_login_time else "",
            "last_quota_refresh_time": (
                state.last_quota_refresh_time.isoformat() if state.last_quota_refresh_time else ""
            ),
            "last_login_result": state.last_login_result,
            "last_login_message": state.last_login_message,
        }
        write_json_atomic(self._state_path, payload)

    @staticmethod
    def _load_from_path(path) -> AppState:
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return AppState()

        raw_last_login_time = str(payload.get("last_login_time", "")).strip()
        last_login_time: datetime | None = None
        if raw_last_login_time:
            try:
                last_login_time = datetime.fromisoformat(raw_last_login_time)
            except ValueError:
                last_login_time = None

        raw_last_quota_refresh_time = str(payload.get("last_quota_refresh_time", "")).strip()
        last_quota_refresh_time: datetime | None = None
        if raw_last_quota_refresh_time:
            try:
                last_quota_refresh_time = datetime.fromisoformat(raw_last_quota_refresh_time)
            except ValueError:
                last_quota_refresh_time = None

        return AppState(
            last_login_time=last_login_time,
            last_quota_refresh_time=last_quota_refresh_time,
            last_login_result=str(payload.get("last_login_result", "未执行")).strip() or "未执行",
            last_login_message=str(payload.get("last_login_message", "-")).strip() or "-",
        )
