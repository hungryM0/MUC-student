from __future__ import annotations

import asyncio
from typing import Callable

from PySide6.QtCore import QThread, Signal


class _TaskThread(QThread):
    success = Signal(object)
    failure = Signal(str)

    def __init__(self, task: Callable[[], object], parent=None) -> None:
        super().__init__(parent)
        self._task = task

    def run(self) -> None:
        if self.isInterruptionRequested():
            return
        try:
            result = self._task()
            if asyncio.iscoroutine(result):
                result = asyncio.run(result)
            if not self.isInterruptionRequested():
                self.success.emit(result)
        except Exception as exc:
            if not self.isInterruptionRequested():
                self.failure.emit(str(exc))


class AsyncTaskRunner:
    def __init__(self, parent=None) -> None:
        self._parent = parent
        self._threads: set[_TaskThread] = set()
        self._shutting_down = False

    def run(
        self,
        task: Callable[[], object],
        on_success: Callable[[object], None],
        on_failure: Callable[[str], None],
    ) -> None:
        if self._shutting_down:
            return
        thread = _TaskThread(task, self._parent)
        self._threads.add(thread)

        def cleanup() -> None:
            self._threads.discard(thread)
            thread.deleteLater()

        thread.success.connect(on_success)
        thread.failure.connect(on_failure)
        thread.finished.connect(cleanup)
        thread.start()

    def shutdown(self) -> None:
        self._shutting_down = True
        for thread in list(self._threads):
            if not thread.isRunning():
                continue
            thread.requestInterruption()
            thread.wait(3000)
