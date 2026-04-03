from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(slots=True)
class AccountOptionViewModel:
    account_id: str
    label: str


@dataclass(slots=True)
class LoginSummaryViewModel:
    result_text: str
    login_time_text: str
    message: str


@dataclass(slots=True)
class QuotaCardViewModel:
    remark_name: str | None
    used_traffic_text: str
    product_balance_text: str
    online_device_count_text: str
    included_package_text: str
    progress_percent: float | None
    loading: bool


@dataclass(slots=True)
class HomePageViewModel:
    ip: str
    login_time_text: str
    accounts: list[AccountOptionViewModel] = field(default_factory=list)
    selected_account_id: str = ""
    login_button_mode: str = "start"
    login_summary: LoginSummaryViewModel = field(
        default_factory=lambda: LoginSummaryViewModel("未执行", "-", "-")
    )
    quota_card: QuotaCardViewModel = field(
        default_factory=lambda: QuotaCardViewModel(None, "-", "-", "-", "", None, False)
    )


@dataclass(slots=True)
class AccountCardViewModel:
    account_id: str
    remark_name: str
    username: str
    used_traffic_text: str
    product_balance_text: str
    included_package_text: str
    online_device_count_text: str
    package_text: str
    status_text: str
    detail_text: str
    updated_at_text: str
    progress_percent: float | None
    is_current_online_account: bool
    can_logout_local_device: bool
    logout_action_enabled: bool


@dataclass(slots=True)
class PoolQuotaViewModel:
    used_traffic_text: str
    product_balance_text: str
    included_package_text: str
    progress_percent: float | None
    loading: bool


@dataclass(slots=True)
class StatusPageViewModel:
    pool_quota: PoolQuotaViewModel
    cards: list[AccountCardViewModel] = field(default_factory=list)
    refreshing: bool = False


@dataclass(slots=True)
class SettingsViewModel:
    portal_url: str
    traffic_portal_url: str
    minimize_to_tray_on_close: bool
    auto_switch_account_on_traffic_exhausted: bool
