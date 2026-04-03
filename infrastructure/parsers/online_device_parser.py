from __future__ import annotations

import re
from html import unescape as html_unescape
from html.parser import HTMLParser
from urllib.parse import urlsplit

from domain.models.traffic import OnlineDeviceRecord


class _HtmlTableParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.tables: list[list[list[str]]] = []
        self._current_table: list[list[str]] | None = None
        self._current_row: list[str] | None = None
        self._current_cell_parts: list[str] | None = None
        self._table_depth = 0
        self._cell_depth_top_level = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        lower_tag = tag.lower()
        if lower_tag == "table":
            self._table_depth += 1
            if self._table_depth == 1:
                self._current_table = []
            return

        if self._table_depth == 0:
            return

        if lower_tag == "tr" and self._table_depth == 1:
            self._current_row = []
            return

        if lower_tag in {"th", "td"} and self._table_depth == 1 and self._current_row is not None:
            self._cell_depth_top_level += 1
            if self._cell_depth_top_level == 1:
                self._current_cell_parts = []

    def handle_data(self, data: str) -> None:
        if self._table_depth == 1 and self._cell_depth_top_level > 0 and self._current_cell_parts is not None:
            self._current_cell_parts.append(data)

    def handle_endtag(self, tag: str) -> None:
        lower_tag = tag.lower()
        if lower_tag in {"th", "td"} and self._table_depth == 1 and self._cell_depth_top_level > 0:
            self._cell_depth_top_level -= 1
            if self._cell_depth_top_level == 0 and self._current_row is not None and self._current_cell_parts is not None:
                cell_text = " ".join("".join(self._current_cell_parts).split())
                self._current_row.append(cell_text)
                self._current_cell_parts = None
            return

        if lower_tag == "tr" and self._table_depth == 1 and self._current_table is not None and self._current_row is not None:
            if any(cell for cell in self._current_row):
                self._current_table.append(self._current_row)
            self._current_row = None
            return

        if lower_tag == "table" and self._table_depth > 0:
            if self._table_depth == 1 and self._current_table is not None:
                if self._current_table:
                    self.tables.append(self._current_table)
                self._current_table = None
            self._table_depth -= 1


def clean_html_text(raw_html: str) -> str:
    text = re.sub(r"<[^>]+>", " ", raw_html or "")
    text = html_unescape(text)
    return " ".join(text.split()).strip()


def extract_table_cell_by_seq(row_html: str, col_seq: str) -> str:
    pattern = re.compile(
        rf'<td[^>]*data-col-seq=["\']{re.escape(col_seq)}["\'][^>]*>(.*?)</td>',
        flags=re.IGNORECASE | re.DOTALL,
    )
    match = pattern.search(row_html or "")
    if not match:
        return ""
    return clean_html_text(match.group(1))


def extract_logout_href(row_html: str) -> str:
    match = re.search(
        r'<a[^>]*href=["\'](?P<href>[^"\']*?/home/delete[^"\']*)["\']',
        row_html or "",
        flags=re.IGNORECASE | re.DOTALL,
    )
    if not match:
        return ""
    return html_unescape(match.group("href")).strip()


def normalize_logout_path(raw_href: str) -> str:
    if not raw_href:
        return ""
    parsed = urlsplit(raw_href)
    path = parsed.path or ""
    if parsed.query:
        path = f"{path}?{parsed.query}"
    if not path:
        return ""
    if not path.startswith("/"):
        return "/" + path.lstrip("/")
    return path


def parse_online_devices(html: str) -> list[OnlineDeviceRecord]:
    records: list[OnlineDeviceRecord] = []
    row_pattern = re.compile(
        r'<tr[^>]*data-key=["\'](?P<device_id>[^"\']+)["\'][^>]*>(?P<body>.*?)</tr>',
        flags=re.IGNORECASE | re.DOTALL,
    )
    for match in row_pattern.finditer(html or ""):
        row_html = match.group(0)
        if "/home/delete" not in row_html.lower():
            continue

        body_html = match.group("body")
        ip_text = extract_table_cell_by_seq(body_html, "1")
        logout_href = extract_logout_href(row_html)
        if not ip_text or not logout_href:
            continue

        device_id = html_unescape(match.group("device_id")).strip()
        logout_path = normalize_logout_path(logout_href)
        if not device_id or not logout_path:
            continue

        records.append(OnlineDeviceRecord(ip=ip_text, device_id=device_id, logout_path=logout_path))
    return records
