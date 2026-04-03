from __future__ import annotations

from dataclasses import dataclass

from domain.models.app_state import AppState
from domain.models.preferences import UserPreferences
from infrastructure.persistence.account_store_repository import AccountBootstrapResult, AccountStoreRepository
from infrastructure.persistence.app_state_repository import AppStateRepository
from infrastructure.persistence.user_preferences_repository import UserPreferencesRepository


@dataclass(slots=True)
class LoadedDashboardState:
    bootstrap: AccountBootstrapResult
    app_state: AppState
    preferences: UserPreferences


class LoadDashboardStateUseCase:
    def __init__(
        self,
        account_repo: AccountStoreRepository,
        app_state_repo: AppStateRepository,
        preferences_repo: UserPreferencesRepository,
    ) -> None:
        self._account_repo = account_repo
        self._app_state_repo = app_state_repo
        self._preferences_repo = preferences_repo

    def execute(self) -> LoadedDashboardState:
        return LoadedDashboardState(
            bootstrap=self._account_repo.ensure_store(),
            app_state=self._app_state_repo.load_state(),
            preferences=self._preferences_repo.load_preferences(),
        )
