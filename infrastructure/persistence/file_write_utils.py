from __future__ import annotations

import json
import os
from contextlib import suppress
from pathlib import Path
from typing import Any


def write_json_atomic(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp_path = path.with_name(f".{path.name}.tmp")
    content = json.dumps(payload, ensure_ascii=False, indent=2)

    try:
        temp_path.write_text(content, encoding="utf-8")
        os.replace(temp_path, path)
    finally:
        with suppress(OSError):
            if temp_path.exists():
                temp_path.unlink()
