from __future__ import annotations

from html import escape

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import QHBoxLayout, QVBoxLayout, QWidget
from qfluentwidgets.common.style_sheet import themeColor
from qfluentwidgets.components.widgets.button import PrimaryPushButton
from qfluentwidgets.components.widgets.card_widget import HeaderCardWidget
from qfluentwidgets.components.widgets.combo_box import ComboBox
from qfluentwidgets.components.widgets.label import (
    BodyLabel,
    CaptionLabel,
    StrongBodyLabel,
    SubtitleLabel,
)
from qfluentwidgets.components.widgets.line_edit import PlainTextEdit
from qfluentwidgets.components.widgets.progress_bar import (
    IndeterminateProgressBar,
    ProgressBar,
)
from qfluentwidgets.components.widgets.progress_ring import IndeterminateProgressRing

from application.view_models import HomePageViewModel
from ui.app_text import APP_NAME


class QuotaCard(HeaderCardWidget):
    _COLOR_WARNING = QColor("#f2b01e")
    _COLOR_DANGER = QColor("#f57c00")
    _COLOR_CRITICAL = QColor("#d13438")

    # qfluentwidgets stubs type HeaderCardWidget.__init__ as singledispatchmethod,
    # which triggers a false-positive override error in Pylance.
    def __init__(self, parent: QWidget | None = None) -> None:  # type: ignore[override]
        super().__init__(parent)
        self.setTitle("账号配额")

        self.name_label = StrongBodyLabel("", self)
        self.name_label.setTextColor(QColor("#202020"), QColor("#202020"))

        self.indeterminate_bar = IndeterminateProgressBar(self)
        self.indeterminate_bar.start()

        self.progress_bar = ProgressBar(self)
        self.progress_bar.setRange(0, 1000)
        self.progress_bar.hide()

        self.online_device_label = BodyLabel("", self)
        self.online_device_label.setTextColor(QColor("#202020"), QColor("#202020"))
        self.used_traffic_label = BodyLabel("", self)
        self.used_traffic_label.setTextColor(QColor("#202020"), QColor("#202020"))
        self.total_traffic_label = BodyLabel("", self)
        self.total_traffic_label.setTextColor(QColor("#202020"), QColor("#202020"))
        self.total_traffic_label.setTextFormat(Qt.TextFormat.RichText)

        self.loading_tip_label = CaptionLabel("", self)
        self.loading_tip_label.setTextColor(QColor("#606060"), QColor("#606060"))
        self.loading_tip_label.hide()

        inner = QVBoxLayout()
        inner.setSpacing(8)
        inner.setContentsMargins(0, 0, 0, 0)
        inner.addWidget(self.name_label)
        inner.addWidget(self.indeterminate_bar)
        inner.addWidget(self.progress_bar)
        inner.addWidget(self.online_device_label)
        inner.addWidget(self.used_traffic_label)
        inner.addWidget(self.total_traffic_label)
        inner.addWidget(self.loading_tip_label)
        self.viewLayout.addLayout(inner)

    def _set_loading(self, loading: bool) -> None:
        if loading:
            self.indeterminate_bar.show()
            self.indeterminate_bar.start()
            self.progress_bar.hide()
            self.online_device_label.setText("在线设备数：-")
            self.used_traffic_label.setText("已用流量：正在加载...")
            self.total_traffic_label.setText("账户总流量：正在加载...")
            self.loading_tip_label.setText("正在加载详细配额...")
            self.loading_tip_label.show()
        else:
            self.indeterminate_bar.stop()
            self.indeterminate_bar.hide()
            self.progress_bar.show()
            self.loading_tip_label.hide()

    def update_data(
        self,
        remark_name: str,
        used_traffic_text: str,
        product_balance_text: str,
        online_device_count_text: str,
        included_package_text: str,
        progress_percent: float | None,
        loading: bool,
    ) -> None:
        self.name_label.setText(remark_name)
        if loading:
            self._set_loading(True)
            return

        self._set_loading(False)
        if progress_percent is not None:
            self.progress_bar.setValue(int(round(progress_percent * 10)))
        else:
            self.progress_bar.setValue(0)
        color = self._resolve_color(progress_percent)
        self.progress_bar.setCustomBarColor(color, color)

        online_text = (online_device_count_text or "").strip()
        self.online_device_label.setText(f"在线设备数：{online_text or '-'}")

        self.used_traffic_label.setText(f"已用流量：{used_traffic_text}")

        package_text = (included_package_text or "").strip()
        self.total_traffic_label.setText(self._build_total_traffic_rich_text(product_balance_text, package_text))

    def _resolve_color(self, progress_percent: float | None) -> QColor:
        if progress_percent is None:
            return themeColor()
        if progress_percent <= 50:
            return themeColor()
        if progress_percent <= 75:
            return self._COLOR_WARNING
        if progress_percent <= 90:
            return self._COLOR_DANGER
        return self._COLOR_CRITICAL

    @staticmethod
    def _build_total_traffic_rich_text(product_balance_text: str, included_package_text: str) -> str:
        total_text = escape(product_balance_text or "-")
        if not included_package_text:
            return f"账户总流量：{total_text}"

        package_text = escape(f"[{included_package_text}]")
        return (
            f"账户总流量：{total_text} "
            f"<span style=\"color: #16a34a;\">{package_text}</span>"
        )


class HomePage(QWidget):
    relogin_requested = Signal()
    selected_account_changed = Signal(str)

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setObjectName("home-page")
        self._login_mode = "start"
        self._has_accounts = False
        self._build_ui()

    def _build_ui(self) -> None:
        root = QVBoxLayout(self)
        root.setContentsMargins(24, 24, 24, 24)
        root.setSpacing(12)

        title = SubtitleLabel(APP_NAME)
        title.setTextColor(QColor("#202020"), QColor("#202020"))

        self.ip_label = BodyLabel("当前内网 IPv4：unknown")
        self.ip_label.setTextColor(QColor("#202020"), QColor("#202020"))
        self.login_time_label = BodyLabel("最近一次登录时间：-")
        self.login_time_label.setTextColor(QColor("#202020"), QColor("#202020"))

        account_row = QHBoxLayout()
        account_label = BodyLabel("登录账号")
        account_label.setTextColor(QColor("#202020"), QColor("#202020"))

        self.account_combo = ComboBox(self)
        self.account_combo.setMinimumWidth(320)
        self.account_combo.setPlaceholderText("请先在状态页添加账号")
        self.account_combo.currentIndexChanged.connect(self._on_account_index_changed)

        account_row.addWidget(account_label)
        account_row.addWidget(self.account_combo)
        account_row.addStretch(1)

        button_row = QHBoxLayout()
        self.relogin_btn = PrimaryPushButton("开始登录")
        self.relogin_btn.clicked.connect(self.relogin_requested.emit)
        button_row.addWidget(self.relogin_btn)
        self.login_loading_ring = IndeterminateProgressRing(self)
        self.login_loading_ring.setFixedSize(20, 20)
        self.login_loading_ring.setStrokeWidth(3)
        self.login_loading_ring.hide()
        button_row.addWidget(self.login_loading_ring)
        button_row.addStretch(1)

        self.quota_card = QuotaCard(self)
        self.quota_card.hide()

        log_title = SubtitleLabel("登录日志")
        log_title.setTextColor(QColor("#202020"), QColor("#202020"))
        self.log_text = PlainTextEdit()
        self.log_text.setReadOnly(True)
        self.log_text.setPlaceholderText("暂无日志")

        root.addWidget(title)
        root.addWidget(self.ip_label)
        root.addWidget(self.login_time_label)
        root.addLayout(account_row)
        root.addLayout(button_row)
        root.addWidget(self.quota_card)
        root.addWidget(log_title)
        root.addWidget(self.log_text, 1)

    def update_login_summary(self, _result_text: str, login_time_text: str, _message: str) -> None:
        self.login_time_label.setText(f"最近一次登录时间：{login_time_text}")

    def update_status_summary(self, ip: str, login_time_text: str) -> None:
        self.ip_label.setText(f"当前内网 IPv4：{ip}")
        self.login_time_label.setText(f"最近一次登录时间：{login_time_text}")

    def set_accounts(self, accounts: list[tuple[str, str]], selected_account_id: str) -> None:
        self.account_combo.blockSignals(True)
        self.account_combo.clear()

        selected_index = -1
        for index, account in enumerate(accounts):
            account_id, account_label = account
            self.account_combo.addItem(account_label, userData=account_id)
            if account_id == selected_account_id:
                selected_index = index

        self._has_accounts = bool(accounts)
        if selected_index >= 0:
            self.account_combo.setCurrentIndex(selected_index)
            auto_selected_index = -1
            auto_selected_account_id = ""
        else:
            auto_selected_index = 0 if accounts else -1
            auto_selected_account_id = str(accounts[0][0]) if accounts else ""
            self.account_combo.setCurrentIndex(auto_selected_index)
        self.account_combo.blockSignals(False)
        if auto_selected_index >= 0 and auto_selected_account_id:
            self.selected_account_changed.emit(auto_selected_account_id)
        self._update_login_controls()

    def update_quota_card(
        self,
        remark_name: str | None,
        used_traffic_text: str,
        product_balance_text: str,
        online_device_count_text: str,
        included_package_text: str,
        progress_percent: float | None,
        loading: bool,
    ) -> None:
        if remark_name is None:
            self.quota_card.hide()
            return
        self.quota_card.show()
        self.quota_card.update_data(
            remark_name=remark_name,
            used_traffic_text=used_traffic_text,
            product_balance_text=product_balance_text,
            online_device_count_text=online_device_count_text,
            included_package_text=included_package_text,
            progress_percent=progress_percent,
            loading=loading,
        )

    def append_log(self, line: str) -> None:
        self.log_text.appendPlainText(line)

    def set_login_button_mode(self, mode: str) -> None:
        self._login_mode = mode
        self._update_login_controls()

    def apply_view_model(self, view_model: HomePageViewModel) -> None:
        self.update_status_summary(view_model.ip, view_model.login_time_text)
        self.set_accounts(
            [(account.account_id, account.label) for account in view_model.accounts],
            view_model.selected_account_id,
        )
        self.set_login_button_mode(view_model.login_button_mode)
        self.update_login_summary(
            view_model.login_summary.result_text,
            view_model.login_summary.login_time_text,
            view_model.login_summary.message,
        )
        self.update_quota_card(
            remark_name=view_model.quota_card.remark_name,
            used_traffic_text=view_model.quota_card.used_traffic_text,
            product_balance_text=view_model.quota_card.product_balance_text,
            online_device_count_text=view_model.quota_card.online_device_count_text,
            included_package_text=view_model.quota_card.included_package_text,
            progress_percent=view_model.quota_card.progress_percent,
            loading=view_model.quota_card.loading,
        )

    def _update_login_controls(self) -> None:
        has_available_account = self._has_accounts and self.account_combo.currentIndex() >= 0
        self.account_combo.setEnabled(self._login_mode != "running" and self._has_accounts)

        if self._login_mode == "running":
            self.relogin_btn.setText("登录中...")
            self.relogin_btn.setEnabled(False)
            self.login_loading_ring.show()
            self.login_loading_ring.start()
            return

        self.login_loading_ring.stop()
        self.login_loading_ring.hide()

        if not has_available_account:
            self.relogin_btn.setText("请先添加账号")
            self.relogin_btn.setEnabled(False)
            return

        self.relogin_btn.setText("开始登录")
        self.relogin_btn.setEnabled(True)

    def _on_account_index_changed(self, index: int) -> None:
        if index < 0:
            return
        account_id = self.account_combo.itemData(index)
        if account_id:
            self.selected_account_changed.emit(str(account_id))
        self._update_login_controls()

