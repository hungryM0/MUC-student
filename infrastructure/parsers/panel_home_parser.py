from __future__ import annotations

import re

from domain.models.traffic import OnlineDeviceRecord
from infrastructure.parsers.online_device_parser import parse_online_devices


BASE_PRODUCT_BALANCE_GB = 45.0


def parse_home_table(html: str) -> tuple[str, str, str, str]:
    parser = _HtmlTableParser()
    parser.feed(html)
    parser.close()

    required_headers = {"产品名称", "计费策略", "已用流量", "产品余额"}
    for table in parser.tables:
        header_index = match_header_row(table, required_headers)
        if header_index < 0:
            continue

        header_row = table[header_index]
        field_indexes = {name: header_row.index(name) for name in required_headers}
        for row in table[header_index + 1 :]:
            if len(row) <= max(field_indexes.values()):
                continue
            package_name = row[field_indexes["产品名称"]]
            billing_policy = row[field_indexes["计费策略"]]
            used_traffic = row[field_indexes["已用流量"]]
            product_balance = row[field_indexes["产品余额"]]
            if package_name or used_traffic or product_balance:
                return (
                    package_name or "未知套餐",
                    billing_policy or "-",
                    used_traffic or "-",
                    product_balance or "-",
                )

    raise RuntimeError("登录成功了，但没在 /home 里找到流量表格")


class _HtmlTableParser:
    def __init__(self) -> None:
        from infrastructure.parsers.online_device_parser import _HtmlTableParser as BaseParser
        self._delegate = BaseParser()
        self.tables = self._delegate.tables

    def feed(self, html: str) -> None:
        self._delegate.feed(html)

    def close(self) -> None:
        self._delegate.close()


def match_header_row(table: list[list[str]], required_headers: set[str]) -> int:
    for idx, row in enumerate(table):
        row_set = set(cell.strip() for cell in row if cell.strip())
        if required_headers.issubset(row_set):
            return idx
    return -1


def build_product_balance_texts(html: str) -> tuple[str, str]:
    package_total_gb = 0.0
    matches = re.findall(
        r"可用流量[:：]\s*([0-9]+(?:\.[0-9]+)?)\s*([KMGT])B?",
        html or "",
        flags=re.IGNORECASE,
    )
    for value_text, unit in matches:
        package_total_gb += convert_to_gigabytes(float(value_text), unit.upper())

    total_gb = BASE_PRODUCT_BALANCE_GB + package_total_gb
    included_package_text = ""
    if package_total_gb > 0:
        included_package_text = f"含{package_total_gb:.2f}GB套餐流量"

    return f"{total_gb:.2f}GB", included_package_text


def extract_csrf_meta(html: str) -> tuple[str, str]:
    from infrastructure.parsers.portal_page_parser import extract_meta_content
    csrf_param = extract_meta_content(html, "csrf-param")
    csrf_token = extract_meta_content(html, "csrf-token")
    return csrf_param, csrf_token


def match_local_ip_device(online_devices: list[OnlineDeviceRecord], local_ip: str | None) -> OnlineDeviceRecord | None:
    local_ip_text = (local_ip or "").strip()
    if not local_ip_text or local_ip_text == "unknown":
        return None
    for record in online_devices:
        if record.ip.strip() == local_ip_text:
            return record
    return None


def convert_to_gigabytes(value: float, unit: str) -> float:
    if unit == "K":
        return value / 1024 / 1024
    if unit == "M":
        return value / 1024
    if unit == "G":
        return value
    if unit == "T":
        return value * 1024
    return 0.0


def parse_panel_home(html: str, local_ip: str | None = None) -> tuple[str, str, str, str, list[OnlineDeviceRecord], OnlineDeviceRecord | None]:
    package_name, billing_policy, used_traffic, _ = parse_home_table(html)
    product_balance, included_package_text = build_product_balance_texts(html)
    online_devices = parse_online_devices(html)
    matched = match_local_ip_device(online_devices, local_ip)
    return package_name, billing_policy, used_traffic, product_balance, online_devices, matched
