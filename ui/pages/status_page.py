from __future__ import annotations

from html import escape

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import (
    QFrame,
    QHBoxLayout,
    QVBoxLayout,
    QWidget,
)
from qfluentwidgets.common.icon import FluentIcon as FIF
from qfluentwidgets.common.style_sheet import themeColor
from qfluentwidgets.components.dialog_box.dialog import MessageBox
from qfluentwidgets.components.widgets.button import PrimaryPushButton, PushButton
from qfluentwidgets.components.widgets.card_widget import ElevatedCardWidget
from qfluentwidgets.components.widgets.label import (
    BodyLabel,
    CaptionLabel,
    StrongBodyLabel,
    SubtitleLabel,
)
from qfluentwidgets.components.widgets.progress_bar import (
    IndeterminateProgressBar,
    ProgressBar,
)
from qfluentwidgets.components.widgets.progress_ring import (
    IndeterminateProgressRing,
    ProgressRing,
)
from qfluentwidgets.components.widgets.scroll_area import ScrollArea

from application.view_models import AccountCardViewModel, PoolQuotaViewModel, StatusPageViewModel
from ui.app_text import APP_NAME


def _build_total_traffic_rich_text(
    product_balance_text: str,
    included_package_text: str,
    prefix: str = "账户总流量",
) -> str:
    total_text = escape(product_balance_text or "-")
    if not included_package_text:
        return f"{prefix}：{total_text}"

    package_text = escape(f"[{included_package_text}]")
    return (
        f"{prefix}：{total_text} "
        f"<span style=\"color: #16a34a;\">{package_text}</span>"
    )


class AccountStatusCard(ElevatedCardWidget):
    _RING_WARNING_COLOR = QColor("#f2b01e")
    _RING_DANGER_COLOR = QColor("#f57c00")
    _RING_CRITICAL_COLOR = QColor("#d13438")

    edit_requested = Signal(str)
    delete_requested = Signal(str)
    logout_local_requested = Signal(str)

    def __init__(self, card_data: AccountCardViewModel, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self._account_id = card_data.account_id
        self.setBorderRadius(18)
        self._build_ui(card_data)

    def _build_ui(self, card_data: AccountCardViewModel) -> None:
        root = QHBoxLayout(self)
        root.setContentsMargins(20, 18, 20, 18)
        root.setSpacing(18)

        self.progress_ring = ProgressRing(self)
        self.progress_ring.setFixedSize(88, 88)
        self.progress_ring.setStrokeWidth(8)
        self.progress_ring.setRange(0, 1000)
        self.progress_ring.setTextVisible(True)
        root.addWidget(self.progress_ring, 0, Qt.AlignmentFlag.AlignTop)

        info_layout = QVBoxLayout()
        info_layout.setSpacing(8)

        title_row = QHBoxLayout()
        title_row.setSpacing(8)

        title_label = StrongBodyLabel(card_data.remark_name, self)
        title_label.setTextColor(QColor("#202020"), QColor("#202020"))
        title_row.addWidget(title_label)

        if card_data.is_current_online_account:
            current_login_label = CaptionLabel("（本机在线账号）", self)
            current_login_label.setTextColor(QColor("#0f6cbd"), QColor("#0f6cbd"))
            title_row.addWidget(current_login_label)

        title_row.addStretch(1)

        username_label = BodyLabel(f"账号：{card_data.username}", self)
        username_label.setTextColor(QColor("#202020"), QColor("#202020"))
        online_device_label = BodyLabel(f"在线设备数：{card_data.online_device_count_text}", self)
        online_device_label.setTextColor(QColor("#202020"), QColor("#202020"))
        used_traffic_label = BodyLabel(f"已用流量：{card_data.used_traffic_text}", self)
        used_traffic_label.setTextColor(QColor("#202020"), QColor("#202020"))
        product_balance_label = BodyLabel(self)
        product_balance_label.setText(
            self._build_product_balance_rich_text(
                card_data.product_balance_text,
                card_data.included_package_text,
            )
        )
        product_balance_label.setTextFormat(Qt.TextFormat.RichText)
        product_balance_label.setTextColor(QColor("#202020"), QColor("#202020"))
        updated_label = CaptionLabel(f"更新时间：{card_data.updated_at_text}", self)
        updated_label.setTextColor(QColor("#606060"), QColor("#606060"))

        button_row = QHBoxLayout()
        button_row.setSpacing(8)
        edit_button = PushButton("编辑", self)
        delete_button = PushButton("删除", self)
        if card_data.can_logout_local_device:
            logout_local_button = PrimaryPushButton("下线本机", self)
            logout_local_button.setEnabled(card_data.logout_action_enabled)
            logout_local_button.clicked.connect(lambda: self.logout_local_requested.emit(self._account_id))
            button_row.addWidget(logout_local_button)
        edit_button.clicked.connect(lambda: self.edit_requested.emit(self._account_id))
        delete_button.clicked.connect(lambda: self.delete_requested.emit(self._account_id))
        button_row.addWidget(edit_button)
        button_row.addWidget(delete_button)
        button_row.addStretch(1)

        info_layout.addLayout(title_row)
        info_layout.addWidget(username_label)
        info_layout.addWidget(online_device_label)
        info_layout.addWidget(used_traffic_label)
        info_layout.addWidget(product_balance_label)
        info_layout.addWidget(updated_label)
        info_layout.addLayout(button_row)
        root.addLayout(info_layout, 1)

        self._apply_progress_state(card_data.progress_percent)

    def _apply_progress_state(self, progress_percent: float | None) -> None:
        if progress_percent is None:
            self.progress_ring.setValue(0)
            self.progress_ring.setFormat("--")
            self.progress_ring.setTextVisible(True)
            self.progress_ring.setCustomBarColor(themeColor(), themeColor())
            return

        normalized_percent = max(0.0, min(100.0, float(progress_percent)))
        display_percent = round(normalized_percent, 1)
        self.progress_ring.setValue(int(round(display_percent * 10)))
        self.progress_ring.setFormat(f"{display_percent:.1f}%")
        self.progress_ring.setTextVisible(True)
        ring_color = self._resolve_progress_ring_color(normalized_percent)
        self.progress_ring.setCustomBarColor(ring_color, ring_color)

    def _resolve_progress_ring_color(self, progress_percent: float) -> QColor:
        if progress_percent <= 50:
            return themeColor()

        if progress_percent <= 75:
            return self._RING_WARNING_COLOR

        if progress_percent <= 90:
            return self._RING_DANGER_COLOR

        return self._RING_CRITICAL_COLOR

    @staticmethod
    def _build_product_balance_rich_text(
        product_balance_text: str,
        included_package_text: str,
    ) -> str:
        return _build_total_traffic_rich_text(
            product_balance_text=product_balance_text,
            included_package_text=included_package_text,
            prefix="账户总流量",
        )


class StatusPage(QWidget):
    _POOL_COLOR_WARNING = QColor("#f2b01e")
    _POOL_COLOR_DANGER = QColor("#f57c00")
    _POOL_COLOR_CRITICAL = QColor("#d13438")

    add_account_requested = Signal()
    edit_account_requested = Signal(str)
    delete_account_requested = Signal(str)
    logout_local_requested = Signal(str)
    refresh_requested = Signal()

    def __init__(self, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setObjectName("status-page")
        self._build_ui()

    def _build_ui(self) -> None:
        root = QVBoxLayout(self)
        root.setContentsMargins(24, 24, 24, 24)
        root.setSpacing(16)

        title_row = QHBoxLayout()
        title = SubtitleLabel(f"{APP_NAME} 状态")
        title.setTextColor(QColor("#202020"), QColor("#202020"))
        add_account_button = PrimaryPushButton("添加账号", self)
        add_account_button.clicked.connect(self.add_account_requested.emit)
        self.refresh_button = PushButton(FIF.SYNC, "刷新状态", self)
        self.refresh_button.clicked.connect(self.refresh_requested.emit)
        self.refresh_loading_ring = IndeterminateProgressRing(self)
        self.refresh_loading_ring.setFixedSize(18, 18)
        self.refresh_loading_ring.setStrokeWidth(3)
        self.refresh_loading_ring.hide()
        title_row.addWidget(title)
        title_row.addStretch(1)
        title_row.addWidget(self.refresh_button)
        title_row.addWidget(self.refresh_loading_ring)
        title_row.addWidget(add_account_button)

        self.pool_quota_card = ElevatedCardWidget(self)
        self.pool_quota_card.setBorderRadius(18)
        pool_quota_layout = QVBoxLayout(self.pool_quota_card)
        pool_quota_layout.setContentsMargins(20, 16, 20, 16)
        pool_quota_layout.setSpacing(8)
        pool_quota_title = StrongBodyLabel("号池总配额", self.pool_quota_card)
        pool_quota_title.setTextColor(QColor("#202020"), QColor("#202020"))
        self.pool_used_traffic_label = BodyLabel("已用流量：-", self.pool_quota_card)
        self.pool_used_traffic_label.setTextColor(QColor("#202020"), QColor("#202020"))
        self.pool_progress_bar = ProgressBar(self.pool_quota_card)
        self.pool_progress_bar.setRange(0, 1000)
        self.pool_progress_bar.setValue(0)
        self.pool_progress_bar.setCustomBarColor(themeColor(), themeColor())
        self.pool_loading_bar = IndeterminateProgressBar(self.pool_quota_card)
        self.pool_loading_bar.hide()
        self.pool_progress_percent_label = CaptionLabel("--", self.pool_quota_card)
        self.pool_progress_percent_label.setTextColor(QColor("#606060"), QColor("#606060"))

        pool_progress_row = QHBoxLayout()
        pool_progress_row.setContentsMargins(0, 0, 0, 0)
        pool_progress_row.setSpacing(8)
        pool_progress_row.addWidget(self.pool_progress_bar, 1)
        pool_progress_row.addWidget(self.pool_loading_bar, 1)
        pool_progress_row.addWidget(self.pool_progress_percent_label, 0, Qt.AlignmentFlag.AlignRight)

        self.pool_total_traffic_label = BodyLabel(self.pool_quota_card)
        self.pool_total_traffic_label.setTextFormat(Qt.TextFormat.RichText)
        self.pool_total_traffic_label.setTextColor(QColor("#202020"), QColor("#202020"))
        self.pool_total_traffic_label.setText(
            _build_total_traffic_rich_text("-", "", prefix="号池总流量")
        )
        pool_quota_layout.addWidget(pool_quota_title)
        pool_quota_layout.addLayout(pool_progress_row)
        pool_quota_layout.addWidget(self.pool_used_traffic_label)
        pool_quota_layout.addWidget(self.pool_total_traffic_label)

        self.empty_label = BodyLabel("还没有账号，先点右上角“添加账号”。", self)
        self.empty_label.setTextColor(QColor("#606060"), QColor("#606060"))

        self.cards_container = QWidget(self)
        self.cards_container.setObjectName("status-cards-container")
        self.cards_container.setStyleSheet(
            "#status-cards-container { background: transparent; }"
        )
        self.cards_layout = QVBoxLayout(self.cards_container)
        self.cards_layout.setContentsMargins(0, 0, 0, 0)
        self.cards_layout.setSpacing(12)

        scroll_area = ScrollArea(self)
        scroll_area.setWidgetResizable(True)
        scroll_area.setFrameShape(QFrame.Shape.NoFrame)
        scroll_area.enableTransparentBackground()
        scroll_area.setWidget(self.cards_container)

        root.addLayout(title_row)
        root.addWidget(self.pool_quota_card)
        root.addWidget(self.empty_label)
        root.addWidget(scroll_area, 1)

    def set_pool_quota_summary(self, summary: PoolQuotaViewModel) -> None:
        if summary.loading:
            self.pool_progress_bar.hide()
            self.pool_loading_bar.show()
            self.pool_loading_bar.start()
            self.pool_progress_percent_label.setText("加载中...")
        else:
            self.pool_loading_bar.stop()
            self.pool_loading_bar.hide()
            self.pool_progress_bar.show()

        if summary.progress_percent is None:
            self.pool_progress_bar.setValue(0)
            self.pool_progress_bar.setCustomBarColor(themeColor(), themeColor())
            if not summary.loading:
                self.pool_progress_percent_label.setText("--")
        else:
            normalized_percent = max(0.0, min(100.0, float(summary.progress_percent)))
            self.pool_progress_bar.setValue(int(round(normalized_percent * 10)))
            progress_color = self._resolve_pool_progress_color(normalized_percent)
            self.pool_progress_bar.setCustomBarColor(progress_color, progress_color)
            if not summary.loading:
                self.pool_progress_percent_label.setText(f"{normalized_percent:.1f}%")

        self.pool_used_traffic_label.setText(f"已用流量：{summary.used_traffic_text}")
        self.pool_total_traffic_label.setText(
            _build_total_traffic_rich_text(
                product_balance_text=summary.product_balance_text,
                included_package_text=summary.included_package_text,
                prefix="号池总流量",
            )
        )

    def _resolve_pool_progress_color(self, progress_percent: float) -> QColor:
        if progress_percent <= 50:
            return themeColor()
        if progress_percent <= 75:
            return self._POOL_COLOR_WARNING
        if progress_percent <= 90:
            return self._POOL_COLOR_DANGER
        return self._POOL_COLOR_CRITICAL

    def set_accounts(self, cards: list[AccountCardViewModel]) -> None:
        self._clear_cards()
        self.empty_label.setVisible(not cards)

        for card_data in cards:
            card = AccountStatusCard(card_data, self.cards_container)
            card.edit_requested.connect(self.edit_account_requested.emit)
            card.delete_requested.connect(self._confirm_delete_account)
            card.logout_local_requested.connect(self._confirm_logout_local_device)
            self.cards_layout.addWidget(card)

        self.cards_layout.addStretch(1)

    def set_refreshing(self, refreshing: bool) -> None:
        if refreshing:
            self.refresh_button.setText("刷新中...")
            self.refresh_button.setEnabled(False)
            self.refresh_loading_ring.show()
            self.refresh_loading_ring.start()
            return

        self.refresh_button.setText("刷新状态")
        self.refresh_button.setEnabled(True)
        self.refresh_loading_ring.stop()
        self.refresh_loading_ring.hide()

    def _confirm_delete_account(self, account_id: str) -> None:
        message_box = MessageBox(
            "删除账号",
            "确定删除这个账号吗？删掉后需要重新添加才能再用。",
            self,
        )
        message_box.yesButton.setText("删除")
        message_box.cancelButton.setText("取消")
        if message_box.exec():
            self.delete_account_requested.emit(account_id)

    def _confirm_logout_local_device(self, account_id: str) -> None:
        message_box = MessageBox(
            "下线本机设备",
            "下线后本机将会断网，需要重新登录认证。该账号上其他在线设备不受影响\n确认继续吗？",
            self,
        )
        message_box.yesButton.setText("确认下线")
        message_box.cancelButton.setText("取消")
        if message_box.exec():
            self.logout_local_requested.emit(account_id)

    def _clear_cards(self) -> None:
        while self.cards_layout.count():
            item = self.cards_layout.takeAt(0)
            if item is not None:
                widget = item.widget()
                if widget is not None:
                    widget.deleteLater()

    def apply_view_model(self, view_model: StatusPageViewModel) -> None:
        self.set_pool_quota_summary(view_model.pool_quota)
        self.set_accounts(view_model.cards)
        self.set_refreshing(view_model.refreshing)

