from __future__ import annotations

import json
from datetime import datetime

from domain.models.app_state import AppState
from domain.models.preferences import UserPreferences
from infrastructure.persistence.file_write_utils import write_json_atomic
from infrastructure.persistence.runtime_paths_provider import RuntimePathsProvider


class AppStateRepository:
    _KEY_MINIMIZE_TO_TRAY_ON_CLOSE = "minimize_to_tray_on_close"
    _KEY_AUTO_SWITCH_ON_TRAFFIC_EXHAUSTED = "auto_switch_account_on_traffic_exhausted"
    _KEY_RECENT_ACCOUNT_IDS = "recent_account_ids"
    _MAX_RECENT_ACCOUNT_IDS = 20

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

    def load_preferences(self) -> UserPreferences:
        payload = self._load_payload()
        return UserPreferences(
            minimize_to_tray_on_close=bool(payload.get(self._KEY_MINIMIZE_TO_TRAY_ON_CLOSE, False)),
            auto_switch_account_on_traffic_exhausted=bool(payload.get(self._KEY_AUTO_SWITCH_ON_TRAFFIC_EXHAUSTED, False)),
            recent_account_ids=self._normalize_recent_account_ids(payload.get(self._KEY_RECENT_ACCOUNT_IDS, [])),
        )

    def save_state(self, state: AppState) -> None:
        payload = self._load_payload()
        payload.update({
            "last_login_time": state.last_login_time.isoformat() if state.last_login_time else "",
            "last_quota_refresh_time": (
                state.last_quota_refresh_time.isoformat() if state.last_quota_refresh_time else ""
            ),
            "last_login_result": state.last_login_result,
            "last_login_message": state.last_login_message,
        })
        write_json_atomic(self._state_path, payload)

    def set_minimize_to_tray_on_close(self, enabled: bool) -> None:
        payload = self._load_payload()
        payload[self._KEY_MINIMIZE_TO_TRAY_ON_CLOSE] = bool(enabled)
        write_json_atomic(self._state_path, payload)

    def set_auto_switch_on_traffic_exhausted(self, enabled: bool) -> None:
        payload = self._load_payload()
        payload[self._KEY_AUTO_SWITCH_ON_TRAFFIC_EXHAUSTED] = bool(enabled)
        write_json_atomic(self._state_path, payload)

    def mark_account_used(self, account_id: str) -> list[str]:
        account_id_text = account_id.strip()
        if not account_id_text:
            return self.get_recent_account_ids()

        reordered = [account_id_text]
        for existed_id in self.get_recent_account_ids():
            if existed_id != account_id_text:
                reordered.append(existed_id)

        self._save_recent_account_ids(reordered)
        return reordered

    def prune_recent_account_ids(self, valid_account_ids: set[str]) -> list[str]:
        kept = [account_id for account_id in self.get_recent_account_ids() if account_id in valid_account_ids]
        self._save_recent_account_ids(kept)
        return kept

    def get_recent_account_ids(self) -> list[str]:
        payload = self._load_payload()
        return self._normalize_recent_account_ids(payload.get(self._KEY_RECENT_ACCOUNT_IDS, []))

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

    def _load_payload(self) -> dict:
        if self._state_path.exists():
            return self._safe_read_payload(self._state_path)
        legacy_path = self._paths.get_legacy_app_state_path()
        if legacy_path.exists():
            return self._safe_read_payload(legacy_path)
        return {}

    @staticmethod
    def _safe_read_payload(path) -> dict:
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return {}
        return payload if isinstance(payload, dict) else {}

    def _save_recent_account_ids(self, account_ids: list[str]) -> None:
        payload = self._load_payload()
        payload[self._KEY_RECENT_ACCOUNT_IDS] = self._normalize_recent_account_ids(account_ids)
        write_json_atomic(self._state_path, payload)

    def _normalize_recent_account_ids(self, raw_value: object) -> list[str]:
        if isinstance(raw_value, str):
            candidates = [raw_value]
        elif isinstance(raw_value, list):
            candidates = raw_value
        else:
            candidates = []

        normalized: list[str] = []
        seen: set[str] = set()
        for item in candidates:
            account_id = str(item).strip()
            if not account_id or account_id in seen:
                continue
            seen.add(account_id)
            normalized.append(account_id)
            if len(normalized) >= self._MAX_RECENT_ACCOUNT_IDS:
                break
        return normalized
