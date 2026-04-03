from __future__ import annotations


class TrayController:
    def __init__(self, minimize_to_tray_on_close: bool) -> None:
        self._minimize_to_tray_on_close = minimize_to_tray_on_close
        self._quitting_from_tray = False
        self._tray_hint_shown = False

    @property
    def minimize_to_tray_on_close(self) -> bool:
        return self._minimize_to_tray_on_close

    @property
    def quitting_from_tray(self) -> bool:
        return self._quitting_from_tray

    @property
    def tray_hint_shown(self) -> bool:
        return self._tray_hint_shown

    def set_minimize_to_tray_on_close(self, enabled: bool) -> None:
        self._minimize_to_tray_on_close = bool(enabled)

    def mark_quitting_from_tray(self) -> None:
        self._quitting_from_tray = True
        self._minimize_to_tray_on_close = False

    def mark_tray_hint_shown(self) -> None:
        self._tray_hint_shown = True
