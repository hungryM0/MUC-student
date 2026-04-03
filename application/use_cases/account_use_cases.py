from __future__ import annotations

from infrastructure.persistence.account_store_repository import AccountStoreRepository


class AddAccountUseCase:
    def __init__(self, account_repo: AccountStoreRepository) -> None:
        self._account_repo = account_repo

    def execute(self, remark_name: str, username: str, password: str):
        return self._account_repo.add_account(remark_name=remark_name, username=username, password=password)


class EditAccountUseCase:
    def __init__(self, account_repo: AccountStoreRepository) -> None:
        self._account_repo = account_repo

    def execute(self, account_id: str, remark_name: str, username: str, password: str):
        return self._account_repo.update_account(account_id, remark_name, username, password)


class DeleteAccountUseCase:
    def __init__(self, account_repo: AccountStoreRepository) -> None:
        self._account_repo = account_repo

    def execute(self, account_id: str) -> None:
        self._account_repo.delete_account(account_id)


class SelectAccountUseCase:
    def __init__(self, account_repo: AccountStoreRepository) -> None:
        self._account_repo = account_repo

    def execute(self, account_id: str):
        return self._account_repo.select_account(account_id)
