from __future__ import annotations

import re

from domain.models.account import AccountStore
from domain.models.traffic import AccountTrafficSnapshot


def parse_traffic_text_to_mb(text: str) -> float | None:
    normalized = (text or "").upper().replace(" ", "")
    match = re.search(r"(\d+(?:\.\d+)?)(K|M|G|T|B)(?:YTE|YTES|B)?", normalized)
    if not match:
        return None

    value = float(match.group(1))
    unit = match.group(2)
    if unit == "B":
        return value / 1024 / 1024
    if unit == "K":
        return value / 1024
    if unit == "M":
        return value
    if unit == "G":
        return value * 1024
    if unit == "T":
        return value * 1024 * 1024
    return None


def parse_traffic_text_to_gb(text: str) -> float | None:
    value_mb = parse_traffic_text_to_mb(text)
    if value_mb is None:
        return None
    return value_mb / 1024


def extract_included_package_gb(text: str) -> float | None:
    normalized = (text or "").replace(" ", "")
    match = re.search(r"含([0-9]+(?:\.[0-9]+)?)GB套餐流量", normalized, flags=re.IGNORECASE)
    if not match:
        return None
    return float(match.group(1))


def format_megabytes(value_mb: float) -> str:
    mb = max(0.0, float(value_mb))
    if mb >= 1024 * 1024:
        return f"{mb / 1024 / 1024:.2f}T"
    if mb >= 1024:
        return f"{mb / 1024:.2f}G"
    if mb >= 1:
        return f"{mb:.2f}M"
    return f"{mb * 1024:.2f}K"


def build_pool_quota_summary(
    account_store: AccountStore,
    snapshots: dict[str, AccountTrafficSnapshot],
) -> tuple[str, str, str, float | None]:
    used_total_mb = 0.0
    total_balance_gb = 0.0
    included_package_total_gb = 0.0
    has_used_value = False
    has_total_value = False

    for account in account_store.accounts:
        snapshot = snapshots.get(account.id)
        if snapshot is None:
            continue

        used_mb = parse_traffic_text_to_mb(snapshot.used_traffic_text)
        if used_mb is not None:
            used_total_mb += used_mb
            has_used_value = True

        total_gb = parse_traffic_text_to_gb(snapshot.product_balance_text)
        if total_gb is not None:
            total_balance_gb += total_gb
            has_total_value = True

        included_gb = extract_included_package_gb(snapshot.included_package_text)
        if included_gb is not None:
            included_package_total_gb += included_gb

    used_text = format_megabytes(used_total_mb) if has_used_value else "-"
    total_text = f"{total_balance_gb:.2f}GB" if has_total_value else "-"
    included_text = (
        f"含{included_package_total_gb:.2f}GB套餐流量"
        if has_total_value and included_package_total_gb > 0
        else ""
    )
    progress_percent: float | None = None
    total_balance_mb = total_balance_gb * 1024
    if has_total_value and has_used_value and total_balance_mb > 0:
        progress_percent = round(max(0.0, min(100.0, (used_total_mb / total_balance_mb) * 100)), 1)
    return used_text, total_text, included_text, progress_percent
