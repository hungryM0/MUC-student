from __future__ import annotations

from app.container import AppContainer, build_container
from ui.main_window import MainWindow


def build_main_window() -> MainWindow:
    container: AppContainer = build_container()
    return MainWindow(container)
