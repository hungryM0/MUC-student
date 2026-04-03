from __future__ import annotations

import sys
from pathlib import Path


class RuntimePathsProvider:
    def __init__(self) -> None:
        self._legacy_root = Path(__file__).resolve().parents[2]

    def get_app_data_dir(self) -> Path:
        if getattr(sys, "frozen", False):
            target_dir = Path(sys.executable).resolve().parent
        else:
            target_dir = self._legacy_root
        target_dir.mkdir(parents=True, exist_ok=True)
        return target_dir

    def get_accounts_path(self) -> Path:
        return self.get_app_data_dir() / "accounts.json"

    def get_app_state_path(self) -> Path:
        return self.get_app_data_dir() / "app_state.json"

    def get_preferences_dir(self) -> Path:
        return self.get_app_data_dir()

    def get_resource_base_dir(self) -> Path:
        if getattr(sys, "frozen", False):
            meipass = getattr(sys, "_MEIPASS", "")
            if meipass:
                return Path(meipass).resolve()
            return Path(sys.executable).resolve().parent
        return self._legacy_root

    def get_legacy_accounts_path(self) -> Path:
        return self._legacy_root / "accounts.json"

    def get_legacy_app_state_path(self) -> Path:
        return self._legacy_root / "app_state.json"
