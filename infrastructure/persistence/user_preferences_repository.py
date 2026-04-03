from __future__ import annotations

from PySide6.QtCore import QSettings

from domain.models.preferences import UserPreferences
from infrastructure.persistence.runtime_paths_provider import RuntimePathsProvider


class UserPreferencesRepository:
    _KEY_MINIMIZE_TO_TRAY_ON_CLOSE = "behavior/minimize_to_tray_on_close"
    _KEY_AUTO_SWITCH_ON_TRAFFIC_EXHAUSTED = "behavior/auto_switch_account_on_traffic_exhausted"
    _KEY_RECENT_ACCOUNT_IDS = "accounts/recent_account_ids"
    _MAX_RECENT_ACCOUNT_IDS = 20

    def __init__(self, paths: RuntimePathsProvider) -> None:
        preferences_dir = paths.get_preferences_dir()
        self._settings = QSettings(str(preferences_dir / "user_preferences.ini"), QSettings.Format.IniFormat)

    def load_preferences(self) -> UserPreferences:
        return UserPreferences(
            minimize_to_tray_on_close=self._get_bool(self._KEY_MINIMIZE_TO_TRAY_ON_CLOSE, default=False),
            auto_switch_account_on_traffic_exhausted=self._get_bool(
                self._KEY_AUTO_SWITCH_ON_TRAFFIC_EXHAUSTED,
                default=False,
            ),
            recent_account_ids=self.get_recent_account_ids(),
        )

    def set_minimize_to_tray_on_close(self, enabled: bool) -> None:
        self._settings.setValue(self._KEY_MINIMIZE_TO_TRAY_ON_CLOSE, bool(enabled))

    def set_auto_switch_on_traffic_exhausted(self, enabled: bool) -> None:
        self._settings.setValue(self._KEY_AUTO_SWITCH_ON_TRAFFIC_EXHAUSTED, bool(enabled))

    def get_recent_account_ids(self) -> list[str]:
        raw_value = self._settings.value(self._KEY_RECENT_ACCOUNT_IDS, [], type=list)
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
        return normalized

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

    def _save_recent_account_ids(self, account_ids: list[str]) -> None:
        normalized: list[str] = []
        seen: set[str] = set()
        for item in account_ids:
            account_id = str(item).strip()
            if not account_id or account_id in seen:
                continue
            seen.add(account_id)
            normalized.append(account_id)
            if len(normalized) >= self._MAX_RECENT_ACCOUNT_IDS:
                break

        self._settings.setValue(self._KEY_RECENT_ACCOUNT_IDS, normalized)

    def _get_bool(self, key: str, default: bool) -> bool:
        return bool(self._settings.value(key, default, type=bool))
