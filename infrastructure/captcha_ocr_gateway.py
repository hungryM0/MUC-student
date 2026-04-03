from __future__ import annotations

import re
from threading import Lock
from typing import Any


class CaptchaOcrGateway:
    _EXPECTED_CHAR_COUNT = 4

    def __init__(self) -> None:
        self._lock = Lock()
        self._engine: Any = None

    def classification(self, image_bytes: bytes) -> str:
        if not image_bytes:
            return ""
        engine = self._ensure_loaded()
        try:
            raw_text = engine.classification(image_bytes)
        except Exception as exc:
            raise RuntimeError(f"ddddocr 识别失败：{exc}") from exc
        return self._normalize_captcha_code(raw_text)

    def ensure_ready(self) -> None:
        self._ensure_loaded()

    def _ensure_loaded(self) -> Any:
        if self._engine is not None:
            return self._engine
        with self._lock:
            if self._engine is not None:
                return self._engine
            try:
                import ddddocr

                # 不启用 det/ocr 扩展，仅保留最小验证码识别模型。
                self._engine = ddddocr.DdddOcr(show_ad=False)
            except Exception as exc:
                raise RuntimeError(f"ddddocr 初始化失败：{exc}") from exc
            return self._engine

    @classmethod
    def _normalize_captcha_code(cls, raw_text: str | None) -> str:
        normalized = re.sub(r"[^0-9A-Za-z]", "", raw_text or "")
        return normalized[: cls._EXPECTED_CHAR_COUNT]
