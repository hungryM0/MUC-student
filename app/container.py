from __future__ import annotations

from dataclasses import dataclass

from application.dashboard_view_mapper import DashboardViewMapper
from application.services.account_traffic_service import AccountTrafficService
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
from infrastructure.settings import AppSettings
from infrastructure.network.auth_portal_client import AuthPortalClient
from infrastructure.network.http_transport import HttpTransport
from infrastructure.network.network_status_service import NetworkStatusService
from infrastructure.network.self_service_panel_client import SelfServicePanelClient
from infrastructure.captcha_ocr_gateway import CaptchaOcrGateway
from infrastructure.persistence.account_store_repository import AccountStoreRepository
from infrastructure.persistence.app_state_repository import AppStateRepository
from infrastructure.persistence.runtime_paths_provider import RuntimePathsProvider
from ui.controllers.async_task_runner import AsyncTaskRunner
from ui.controllers.main_window_controller import MainWindowController


@dataclass(slots=True)
class AppContainer:
    settings: AppSettings
    controller: MainWindowController


def build_container() -> AppContainer:
    settings = AppSettings.load()
    paths = RuntimePathsProvider()
    account_repo = AccountStoreRepository(paths)
    app_state_repo = AppStateRepository(paths)
    ocr_gateway = CaptchaOcrGateway()
    auth_transport = HttpTransport(settings)
    panel_transport = HttpTransport(settings)
    auth_client = AuthPortalClient(settings, auth_transport, ocr_gateway)
    panel_client = SelfServicePanelClient(settings, panel_transport, ocr_gateway)
    network_status_service = NetworkStatusService(settings)
    traffic_service = AccountTrafficService(panel_client)
    log_service = LogService()
    view_mapper = DashboardViewMapper()
    runner = AsyncTaskRunner()

    controller = MainWindowController(
        settings=settings,
        account_repo=account_repo,
        app_state_repo=app_state_repo,
        load_dashboard_use_case=LoadDashboardStateUseCase(account_repo, app_state_repo),
        add_account_use_case=AddAccountUseCase(account_repo),
        edit_account_use_case=EditAccountUseCase(account_repo),
        delete_account_use_case=DeleteAccountUseCase(account_repo),
        select_account_use_case=SelectAccountUseCase(account_repo),
        login_use_case=LoginSelectedAccountUseCase(
            auth_client,
            panel_client,
            network_status_service,
            traffic_service,
        ),
        refresh_network_use_case=RefreshNetworkStatusUseCase(network_status_service),
        refresh_snapshots_use_case=RefreshAccountSnapshotsUseCase(traffic_service),
        verify_online_account_use_case=VerifyOnlineAccountUseCase(traffic_service),
        logout_local_device_use_case=LogoutLocalDeviceUseCase(network_status_service, panel_client),
        log_service=log_service,
        view_mapper=view_mapper,
        runner=runner,
    )
    return AppContainer(settings=settings, controller=controller)

