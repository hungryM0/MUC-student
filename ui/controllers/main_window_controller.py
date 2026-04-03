from __future__ import annotations

from PySide6.QtCore import QObject, Signal

from application.dashboard_view_mapper import DashboardViewMapper
from application.services.log_service import LogService
from application.use_cases.account_use_cases import (
    AddAccountUseCase,
    DeleteAccountUseCase,
    EditAccountUseCase,
    SelectAccountUseCase,
)
from application.use_cases.load_dashboard_state import LoadDashboardStateUseCase
from application.use_cases.login_selected_account import LoginSelectedAccountUseCase
from application.use_cases.logout_local_device import LogoutLocalDeviceUseCase
from application.use_cases.refresh_use_cases import (
    RefreshAccountSnapshotsUseCase,
    RefreshNetworkStatusUseCase,
    VerifyOnlineAccountUseCase,
)
from domain.models.account import AccountStore, PortalAccount
from domain.models.app_state import AppState
from domain.models.preferences import UserPreferences
from ui.controllers.async_task_runner import AsyncTaskRunner
from ui.controllers.orchestrators import (
    AccountOrchestrator,
    PresentationOrchestrator,
    RefreshOrchestrator,
    SessionOrchestrator,
)


class MainWindowController(QObject):
    home_changed = Signal(object)
    status_changed = Signal(object)
    settings_changed = Signal(object)
    log_arrived = Signal(str)
    warning_requested = Signal(str, str)

    def __init__(
        self,
        *,
        settings,
        account_repo,
        app_state_repo,
        preferences_repo,
        load_dashboard_use_case: LoadDashboardStateUseCase,
        add_account_use_case: AddAccountUseCase,
        edit_account_use_case: EditAccountUseCase,
        delete_account_use_case: DeleteAccountUseCase,
        select_account_use_case: SelectAccountUseCase,
        login_use_case: LoginSelectedAccountUseCase,
        refresh_network_use_case: RefreshNetworkStatusUseCase,
        refresh_snapshots_use_case: RefreshAccountSnapshotsUseCase,
        verify_online_account_use_case: VerifyOnlineAccountUseCase,
        logout_local_device_use_case: LogoutLocalDeviceUseCase,
        log_service: LogService,
        view_mapper: DashboardViewMapper,
        runner: AsyncTaskRunner,
    ) -> None:
        super().__init__()
        self.settings = settings
        self._account_repo = account_repo
        self._app_state_repo = app_state_repo
        self._preferences_repo = preferences_repo
        self._load_dashboard_use_case = load_dashboard_use_case
        self._add_account_use_case = add_account_use_case
        self._edit_account_use_case = edit_account_use_case
        self._delete_account_use_case = delete_account_use_case
        self._select_account_use_case = select_account_use_case
        self._login_use_case = login_use_case
        self._refresh_network_use_case = refresh_network_use_case
        self._refresh_snapshots_use_case = refresh_snapshots_use_case
        self._verify_online_account_use_case = verify_online_account_use_case
        self._logout_local_device_use_case = logout_local_device_use_case
        self._log_service = log_service
        self._view_mapper = view_mapper
        self._runner = runner

        loaded = self._load_dashboard_use_case.execute()
        self.account_store: AccountStore = loaded.bootstrap.store
        self.app_state: AppState = loaded.app_state
        self.preferences: UserPreferences = loaded.preferences
        self._bootstrap_created_file = loaded.bootstrap.created_file

        self._closing = False
        self._login_running = False
        self._local_logout_running = False
        self._status_refresh_running = False
        self._traffic_refresh_running = False
        self._online_account_verify_running = False
        self._pending_online_account_verify = False
        self._current_ip = "unknown"
        self._current_online_account_id = self.account_store.current_online_account_id
        self._status_card_order_snapshot = list(self.account_store.status_card_order_snapshot)
        self._traffic_snapshots = self._view_mapper.restore_cached_snapshots(self.account_store.cached_traffic_snapshots)
        self._recent_account_ids = list(self.preferences.recent_account_ids)

        self._presentation = PresentationOrchestrator(self)
        self._account = AccountOrchestrator(self)
        self._refresh = RefreshOrchestrator(self)
        self._session = SessionOrchestrator(self)

        self._presentation.refresh_status_order_snapshot()
        self._log_service.add_listener(self._presentation.forward_log_to_ui)

    def initialize(self) -> None:
        if self._bootstrap_created_file:
            self._log_service.log("INFO", "已创建空的 accounts.json，现在可以去状态页添加账号")
        self._presentation.emit_all_views()

    def shutdown(self) -> None:
        self._closing = True
        self._runner.shutdown()

    def get_existing_log_lines(self) -> list[str]:
        return self._presentation.get_existing_log_lines()

    def start_login(self) -> None:
        self._session.start_login()

    def refresh_status_page_data(self, force_quota_refresh: bool = False) -> None:
        self._refresh.refresh_status_page_data(force_quota_refresh=force_quota_refresh)

    def refresh_network_status(self) -> None:
        self._refresh.refresh_network_status()

    def refresh_account_snapshots(self, force_refresh: bool = False) -> bool:
        return self._refresh.refresh_account_snapshots(force_refresh=force_refresh)

    def select_account(self, account_id: str) -> None:
        self._account.select_account(account_id)

    def add_account(self, remark_name: str, username: str, password: str) -> None:
        self._account.add_account(remark_name, username, password)

    def get_account(self, account_id: str) -> PortalAccount | None:
        return self._account.get_account(account_id)

    def edit_account(self, account_id: str, remark_name: str, username: str, password: str) -> None:
        self._account.edit_account(account_id, remark_name, username, password)

    def delete_account(self, account_id: str) -> None:
        self._account.delete_account(account_id)

    def logout_local_device_for_account(self, account_id: str) -> None:
        self._session.logout_local_device_for_account(account_id)

    def set_minimize_to_tray_on_close(self, enabled: bool) -> None:
        self._account.set_minimize_to_tray_on_close(enabled)

    def set_auto_switch_account_on_traffic_exhausted(self, enabled: bool) -> None:
        self._account.set_auto_switch_account_on_traffic_exhausted(enabled)
