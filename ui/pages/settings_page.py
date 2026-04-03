from __future__ import annotations

from PySide6.QtCore import Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import QVBoxLayout, QWidget
from qfluentwidgets.common.icon import FluentIcon as FIF
from qfluentwidgets.components.settings.setting_card import SwitchSettingCard
from qfluentwidgets.components.settings.setting_card_group import SettingCardGroup
from qfluentwidgets.components.widgets.card_widget import CardWidget
from qfluentwidgets.components.widgets.label import BodyLabel, CaptionLabel, StrongBodyLabel
from qfluentwidgets.components.widgets.line_edit import LineEdit

from application.view_models import SettingsViewModel

class SettingsSectionCard(CardWidget):
    def __init__(self, title: str, description: str, parent: QWidget | None = None) -> None:
        super().__init__(parent)
        self.setBorderRadius(12)

        root = QVBoxLayout(self)
        root.setContentsMargins(20, 18, 20, 18)
        root.setSpacing(12)

        title_label = StrongBodyLabel(title, self)
        title_label.setTextColor(QColor("#202020"), QColor("#202020"))
        description_label = CaptionLabel(description, self)
        description_label.setTextColor(QColor("#606060"), QColor("#606060"))

        self.content_layout = QVBoxLayout()
        self.content_layout.setContentsMargins(0, 0, 0, 0)
        self.content_layout.setSpacing(12)

        root.addWidget(title_label)
        root.addWidget(description_label)
        root.addLayout(self.content_layout)


class SettingsPage(QWidget):
    minimize_to_tray_changed = Signal(bool)
    auto_switch_account_changed = Signal(bool)

    def __init__(
        self,
        portal_url: str,
        traffic_portal_url: str,
        minimize_to_tray_on_close: bool,
        auto_switch_account_on_traffic_exhausted: bool,
        parent: QWidget | None = None,
    ) -> None:
        super().__init__(parent)
        self.setObjectName("settings-page")
        self._build_ui(
            portal_url,
            traffic_portal_url,
            minimize_to_tray_on_close,
            auto_switch_account_on_traffic_exhausted,
        )

    def _build_ui(
        self,
        portal_url: str,
        traffic_portal_url: str,
        minimize_to_tray_on_close: bool,
        auto_switch_account_on_traffic_exhausted: bool,
    ) -> None:
        root = QVBoxLayout(self)
        root.setContentsMargins(24, 24, 24, 24)
        root.setSpacing(16)

        root.addWidget(
            self._build_behavior_group(
                minimize_to_tray_on_close=minimize_to_tray_on_close,
                auto_switch_account_on_traffic_exhausted=auto_switch_account_on_traffic_exhausted,
            )
        )
        root.addWidget(self._build_portal_card(portal_url, traffic_portal_url))
        root.addStretch(1)

    def _build_behavior_group(
        self,
        minimize_to_tray_on_close: bool,
        auto_switch_account_on_traffic_exhausted: bool,
    ) -> SettingCardGroup:
        group = SettingCardGroup("设置", self)

        self.minimize_to_tray_card = SwitchSettingCard(
            FIF.MINIMIZE,
            "关闭窗口时最小化到托盘",
            "开启后，点右上角关闭不会退出程序，而是最小化到系统托盘。",
            parent=group,
        )
        self.minimize_to_tray_card.setChecked(minimize_to_tray_on_close)
        self._localize_switch_card(self.minimize_to_tray_card)
        self.minimize_to_tray_card.checkedChanged.connect(self.minimize_to_tray_changed.emit)

        self.auto_switch_account_card = SwitchSettingCard(
            FIF.SYNC,
            "流量用完后自动切换账号",
            "优先切到最近使用且未用完流量的账号。",
            parent=group,
        )
        self.auto_switch_account_card.setChecked(auto_switch_account_on_traffic_exhausted)
        self._localize_switch_card(self.auto_switch_account_card)
        self.auto_switch_account_card.checkedChanged.connect(self.auto_switch_account_changed.emit)

        group.addSettingCard(self.minimize_to_tray_card)
        group.addSettingCard(self.auto_switch_account_card)
        return group

    @staticmethod
    def _localize_switch_card(card: SwitchSettingCard) -> None:
        card.switchButton.setOnText("开启")
        card.switchButton.setOffText("关闭")
        card.switchButton.setText("开启" if card.isChecked() else "关闭")
        card.checkedChanged.connect(
            lambda checked, target=card: target.switchButton.setText("开启" if checked else "关闭")
        )

    def _build_portal_card(self, portal_url: str, traffic_portal_url: str) -> SettingsSectionCard:
        card = SettingsSectionCard(
            "接口地址",
            "MUC的校园网区分登录认证和流量查询两个不同的接口地址",
            self,
        )

        portal_label = BodyLabel("认证 URL")
        portal_label.setTextColor(QColor("#202020"), QColor("#202020"))

        self.portal_edit = LineEdit(card)
        self.portal_edit.setText(portal_url)
        self.portal_edit.setReadOnly(True)
        self.portal_edit.setClearButtonEnabled(False)

        traffic_portal_label = BodyLabel("流量查询 URL")
        traffic_portal_label.setTextColor(QColor("#202020"), QColor("#202020"))

        self.traffic_portal_edit = LineEdit(card)
        self.traffic_portal_edit.setText(traffic_portal_url)
        self.traffic_portal_edit.setReadOnly(True)
        self.traffic_portal_edit.setClearButtonEnabled(False)

        card.content_layout.addWidget(portal_label)
        card.content_layout.addWidget(self.portal_edit)
        card.content_layout.addWidget(traffic_portal_label)
        card.content_layout.addWidget(self.traffic_portal_edit)
        return card

    def apply_view_model(self, view_model: SettingsViewModel) -> None:
        self.portal_edit.setText(view_model.portal_url)
        self.traffic_portal_edit.setText(view_model.traffic_portal_url)
        self.minimize_to_tray_card.blockSignals(True)
        self.auto_switch_account_card.blockSignals(True)
        self.minimize_to_tray_card.setChecked(view_model.minimize_to_tray_on_close)
        self.auto_switch_account_card.setChecked(view_model.auto_switch_account_on_traffic_exhausted)
        self.minimize_to_tray_card.switchButton.setText(
            "开启" if view_model.minimize_to_tray_on_close else "关闭"
        )
        self.auto_switch_account_card.switchButton.setText(
            "开启" if view_model.auto_switch_account_on_traffic_exhausted else "关闭"
        )
        self.minimize_to_tray_card.blockSignals(False)
        self.auto_switch_account_card.blockSignals(False)

