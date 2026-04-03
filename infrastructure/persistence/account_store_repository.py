from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from uuid import uuid4

from domain.models.account import AccountStore, CachedTrafficSnapshot, PortalAccount
from infrastructure.persistence.file_write_utils import write_json_atomic
from infrastructure.persistence.runtime_paths_provider import RuntimePathsProvider


@dataclass(slots=True)
class AccountBootstrapResult:
    store: AccountStore
    created_file: bool


class AccountStoreRepository:
    def __init__(self, paths: RuntimePathsProvider) -> None:
        self._paths = paths
        self._accounts_path = paths.get_accounts_path()

    def ensure_store(self) -> AccountBootstrapResult:
        if self._accounts_path.exists():
            store = self.load_store()
            normalized_store = self._normalize_store(store)
            if normalized_store != store:
                self.save_store(normalized_store)
                store = normalized_store
            return AccountBootstrapResult(store=store, created_file=False)

        legacy_store = self._try_load_legacy_store()
        if legacy_store is not None:
            normalized_store = self._normalize_store(legacy_store)
            self.save_store(normalized_store)
            return AccountBootstrapResult(store=normalized_store, created_file=False)

        store = AccountStore()
        self.save_store(store)
        return AccountBootstrapResult(store=store, created_file=True)

    def load_store(self) -> AccountStore:
        if not self._accounts_path.exists():
            legacy_store = self._try_load_legacy_store()
            return legacy_store or AccountStore()
        return self._load_store_from_path(self._accounts_path)

    def save_store(self, store: AccountStore) -> None:
        normalized_store = self._normalize_store(store)
        payload = {
            "selected_account_id": normalized_store.selected_account_id,
            "current_online_account_id": normalized_store.current_online_account_id,
            "status_card_order_snapshot": normalized_store.status_card_order_snapshot,
            "accounts": [
                {
                    "id": account.id,
                    "remark_name": account.remark_name,
                    "username": account.username,
                    "password": account.password,
                }
                for account in normalized_store.accounts
            ],
            "cached_traffic_snapshots": {
                account_id: {
                    "used_traffic_text": snapshot.used_traffic_text,
                    "product_balance_text": snapshot.product_balance_text,
                    "included_package_text": snapshot.included_package_text,
                    "online_device_count_text": snapshot.online_device_count_text,
                    "package_text": snapshot.package_text,
                    "status_text": snapshot.status_text,
                    "detail_text": snapshot.detail_text,
                    "queried_at": snapshot.queried_at.isoformat() if snapshot.queried_at else "",
                    "progress_percent": snapshot.progress_percent,
                }
                for account_id, snapshot in normalized_store.cached_traffic_snapshots.items()
            },
        }
        write_json_atomic(self._accounts_path, payload)

    def add_account(self, remark_name: str, username: str, password: str) -> PortalAccount:
        store = self.load_store()
        account = PortalAccount(
            id=uuid4().hex,
            remark_name=self._require_text(remark_name, "备注名"),
            username=self._require_text(username, "账号"),
            password=self._require_text(password, "密码"),
        )
        self._ensure_username_unique(store.accounts, account.username)
        store.accounts.append(account)
        if not store.selected_account_id:
            store.selected_account_id = account.id
        self.save_store(store)
        return account

    def update_account(self, account_id: str, remark_name: str, username: str, password: str) -> PortalAccount:
        store = self.load_store()
        target_account = self.get_account_by_id(store, account_id)
        if target_account is None:
            raise ValueError("找不到要编辑的账号")

        clean_username = self._require_text(username, "账号")
        self._ensure_username_unique(store.accounts, clean_username, exclude_account_id=account_id)
        target_account.remark_name = self._require_text(remark_name, "备注名")
        target_account.username = clean_username
        target_account.password = self._require_text(password, "密码")
        self.save_store(store)
        return target_account

    def delete_account(self, account_id: str) -> None:
        store = self.load_store()
        remaining_accounts = [account for account in store.accounts if account.id != account_id]
        if len(remaining_accounts) == len(store.accounts):
            raise ValueError("找不到要删除的账号")

        store.accounts = remaining_accounts
        if store.selected_account_id == account_id:
            store.selected_account_id = remaining_accounts[0].id if remaining_accounts else ""
        self.save_store(store)

    def select_account(self, account_id: str) -> PortalAccount:
        store = self.load_store()
        account = self.get_account_by_id(store, account_id)
        if account is None:
            raise ValueError("找不到要选择的账号")

        store.selected_account_id = account.id
        self.save_store(store)
        return account

    def save_cached_traffic_snapshots(
        self,
        cached_snapshots: dict[str, CachedTrafficSnapshot],
        current_online_account_id: str | None = None,
        status_card_order_snapshot: list[str] | None = None,
    ) -> None:
        store = self.load_store()
        valid_account_ids = {account.id for account in store.accounts}
        store.cached_traffic_snapshots = {
            account_id: snapshot
            for account_id, snapshot in cached_snapshots.items()
            if account_id in valid_account_ids
        }
        if current_online_account_id is not None:
            candidate_online_id = current_online_account_id.strip()
            store.current_online_account_id = candidate_online_id if candidate_online_id in valid_account_ids else ""
        if status_card_order_snapshot is not None:
            deduped_order: list[str] = []
            seen_ids: set[str] = set()
            for account_id in status_card_order_snapshot:
                clean_id = str(account_id).strip()
                if not clean_id or clean_id in seen_ids or clean_id not in valid_account_ids:
                    continue
                seen_ids.add(clean_id)
                deduped_order.append(clean_id)
            for account in store.accounts:
                if account.id not in seen_ids:
                    deduped_order.append(account.id)
            store.status_card_order_snapshot = deduped_order
        self.save_store(store)

    @staticmethod
    def get_selected_account(store: AccountStore) -> PortalAccount | None:
        return AccountStoreRepository.get_account_by_id(store, store.selected_account_id)

    @staticmethod
    def get_account_by_id(store: AccountStore, account_id: str) -> PortalAccount | None:
        for account in store.accounts:
            if account.id == account_id:
                return account
        return None

    def _try_load_legacy_store(self) -> AccountStore | None:
        legacy_path = self._paths.get_legacy_accounts_path()
        if not legacy_path.exists():
            return None
        return self._load_store_from_path(legacy_path)

    def _load_store_from_path(self, path: Path) -> AccountStore:
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            raise RuntimeError(f"accounts.json 格式错误：{exc}") from exc

        accounts_payload = payload.get("accounts", [])
        if not isinstance(accounts_payload, list):
            raise RuntimeError("accounts.json 中的 accounts 必须是数组")

        status_card_order_payload = payload.get("status_card_order_snapshot", [])
        if not isinstance(status_card_order_payload, list):
            raise RuntimeError("accounts.json 中的 status_card_order_snapshot 必须是数组")

        cached_snapshots_payload = payload.get("cached_traffic_snapshots", {})
        if not isinstance(cached_snapshots_payload, dict):
            raise RuntimeError("accounts.json 中的 cached_traffic_snapshots 必须是对象")

        accounts: list[PortalAccount] = []
        for raw_account in accounts_payload:
            if not isinstance(raw_account, dict):
                raise RuntimeError("accounts.json 中存在非法账号项")
            account = PortalAccount(
                id=str(raw_account.get("id", "")).strip(),
                remark_name=str(raw_account.get("remark_name", "")).strip(),
                username=str(raw_account.get("username", "")).strip(),
                password=str(raw_account.get("password", "")),
            )
            if not account.id:
                account.id = uuid4().hex
            if not account.remark_name or not account.username:
                continue
            accounts.append(account)

        cached_traffic_snapshots: dict[str, CachedTrafficSnapshot] = {}
        for account_id, raw_snapshot in cached_snapshots_payload.items():
            if not isinstance(raw_snapshot, dict):
                continue
            clean_account_id = str(account_id).strip()
            if not clean_account_id:
                continue
            raw_queried_at = str(raw_snapshot.get("queried_at", "")).strip()
            queried_at: datetime | None = None
            if raw_queried_at:
                try:
                    queried_at = datetime.fromisoformat(raw_queried_at)
                except ValueError:
                    queried_at = None
            cached_traffic_snapshots[clean_account_id] = CachedTrafficSnapshot(
                used_traffic_text=str(raw_snapshot.get("used_traffic_text", "-")).strip() or "-",
                product_balance_text=str(raw_snapshot.get("product_balance_text", "-")).strip() or "-",
                included_package_text=str(raw_snapshot.get("included_package_text", "")).strip(),
                online_device_count_text=str(raw_snapshot.get("online_device_count_text", "-")).strip() or "-",
                package_text=str(raw_snapshot.get("package_text", "-")).strip() or "-",
                status_text=str(raw_snapshot.get("status_text", "已缓存")).strip() or "已缓存",
                detail_text=str(raw_snapshot.get("detail_text", "已沿用上次配额数据（本地缓存）")).strip()
                or "已沿用上次配额数据（本地缓存）",
                queried_at=queried_at,
                progress_percent=self._parse_progress_percent(raw_snapshot.get("progress_percent")),
            )

        store = AccountStore(
            selected_account_id=str(payload.get("selected_account_id", "")).strip(),
            accounts=accounts,
            current_online_account_id=str(payload.get("current_online_account_id", "")).strip(),
            status_card_order_snapshot=[str(item).strip() for item in status_card_order_payload if str(item).strip()],
            cached_traffic_snapshots=cached_traffic_snapshots,
        )
        return self._normalize_store(store)

    @staticmethod
    def _require_text(value: str, field_name: str) -> str:
        clean_value = value.strip()
        if not clean_value:
            raise ValueError(f"{field_name}不能为空")
        return clean_value

    @staticmethod
    def _ensure_username_unique(
        accounts: list[PortalAccount],
        username: str,
        exclude_account_id: str = "",
    ) -> None:
        for account in accounts:
            if account.id == exclude_account_id:
                continue
            if account.username == username:
                raise ValueError("这个账号已经存在了，别重复加")

    @staticmethod
    def _normalize_store(store: AccountStore) -> AccountStore:
        deduplicated_accounts: list[PortalAccount] = []
        seen_ids: set[str] = set()
        for account in store.accounts:
            if account.id in seen_ids:
                continue
            seen_ids.add(account.id)
            deduplicated_accounts.append(account)

        selected_account_id = store.selected_account_id
        if selected_account_id and not any(account.id == selected_account_id for account in deduplicated_accounts):
            selected_account_id = ""
        if not selected_account_id and deduplicated_accounts:
            selected_account_id = deduplicated_accounts[0].id

        valid_account_ids = {account.id for account in deduplicated_accounts}
        normalized_current_online_account_id = (
            store.current_online_account_id if store.current_online_account_id in valid_account_ids else ""
        )
        normalized_status_card_order_snapshot: list[str] = []
        seen_status_order_ids: set[str] = set()
        for account_id in store.status_card_order_snapshot:
            if account_id in seen_status_order_ids or account_id not in valid_account_ids:
                continue
            seen_status_order_ids.add(account_id)
            normalized_status_card_order_snapshot.append(account_id)
        for account in deduplicated_accounts:
            if account.id not in seen_status_order_ids:
                normalized_status_card_order_snapshot.append(account.id)

        normalized_cached_snapshots = {
            account_id: snapshot
            for account_id, snapshot in store.cached_traffic_snapshots.items()
            if account_id in valid_account_ids
        }

        return AccountStore(
            selected_account_id=selected_account_id,
            accounts=deduplicated_accounts,
            current_online_account_id=normalized_current_online_account_id,
            status_card_order_snapshot=normalized_status_card_order_snapshot,
            cached_traffic_snapshots=normalized_cached_snapshots,
        )

    @staticmethod
    def _parse_progress_percent(raw_value: object) -> float | None:
        if isinstance(raw_value, (int, float)):
            return max(0.0, min(100.0, float(raw_value)))
        if isinstance(raw_value, str):
            text = raw_value.strip()
            if not text:
                return None
            try:
                parsed = float(text)
            except ValueError:
                return None
            return max(0.0, min(100.0, parsed))
        return None
