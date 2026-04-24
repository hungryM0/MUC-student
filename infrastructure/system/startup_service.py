from __future__ import annotations

import os
import sys
import winreg
from pathlib import Path

from ui.app_text import APP_NAME


class StartupService:
    _RUN_KEY = r"Software\Microsoft\Windows\CurrentVersion\Run"

    def __init__(self, app_name: str = APP_NAME) -> None:
        self._app_name = app_name

    def set_launch_on_startup(self, enabled: bool) -> None:
        if enabled:
            self._write_startup_entry()
            return
        self._delete_startup_entry()

    def _write_startup_entry(self) -> None:
        with winreg.OpenKey(
            winreg.HKEY_CURRENT_USER,
            self._RUN_KEY,
            0,
            winreg.KEY_SET_VALUE,
        ) as run_key:
            winreg.SetValueEx(run_key, self._app_name, 0, winreg.REG_SZ, self._build_launch_command())

    def _delete_startup_entry(self) -> None:
        with winreg.OpenKey(
            winreg.HKEY_CURRENT_USER,
            self._RUN_KEY,
            0,
            winreg.KEY_SET_VALUE,
        ) as run_key:
            try:
                winreg.DeleteValue(run_key, self._app_name)
            except FileNotFoundError:
                pass

    def _build_launch_command(self) -> str:
        executable_path = Path(sys.executable).resolve()
        if getattr(sys, "frozen", False):
            return self._quote(executable_path)

        script_path = Path(sys.argv[0]).resolve() if sys.argv else Path.cwd() / "main.py"
        if not script_path.exists():
            script_path = Path.cwd() / "main.py"
        return f"{self._quote(executable_path)} {self._quote(script_path)}"

    @staticmethod
    def _quote(path: Path) -> str:
        return f'"{os.fspath(path)}"'
