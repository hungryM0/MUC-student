from __future__ import annotations

from dataclasses import dataclass

from PySide6.QtGui import QColor
from PySide6.QtWidgets import QVBoxLayout, QWidget
from qfluentwidgets.components.dialog_box.message_box_base import MessageBoxBase
from qfluentwidgets.components.widgets.label import BodyLabel, CaptionLabel, SubtitleLabel
from qfluentwidgets.components.widgets.line_edit import LineEdit, PasswordLineEdit

from domain.models.account import PortalAccount


@dataclass(slots=True)
class AccountFormData:
    remark_name: str
    username: str
    password: str


class AccountDialog(MessageBoxBase):
    def __init__(
        self,
        title: str,
        account: PortalAccount | None = None,
        parent: QWidget | None = None,
    ) -> None:
        super().__init__(parent)
        self._dialog_title = title
        self._build_ui(account)

    def _build_ui(self, account: PortalAccount | None) -> None:
        self.title_label = SubtitleLabel(self._dialog_title, self)
        self.title_label.setTextColor(QColor("#202020"), QColor("#202020"))

        self.remark_edit = LineEdit(self)
        self.remark_edit.setPlaceholderText("比如：宿舍主账号")
        self.username_edit = LineEdit(self)
        self.username_edit.setPlaceholderText("请输入校园网账号")
        self.password_edit = PasswordLineEdit(self)
        self.password_edit.setPlaceholderText("请输入密码")

        if account is not None:
            self.remark_edit.setText(account.remark_name)
            self.username_edit.setText(account.username)
            self.password_edit.setText(account.password)

        self.error_label = CaptionLabel("", self)
        self.error_label.setTextColor(QColor("#d13438"), QColor("#d13438"))
        self.error_label.hide()

        self.widget.setMinimumWidth(420)
        self.yesButton.setText("保存")
        self.cancelButton.setText("取消")

        self.viewLayout.addWidget(self.title_label)
        self.viewLayout.addLayout(self._build_field("备注名", self.remark_edit))
        self.viewLayout.addLayout(self._build_field("账号", self.username_edit))
        self.viewLayout.addLayout(self._build_field("密码", self.password_edit))
        self.viewLayout.addWidget(self.error_label)

    def _build_field(self, label_text: str, editor: QWidget) -> QVBoxLayout:
        layout = QVBoxLayout()
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(6)

        label = BodyLabel(label_text, self)
        label.setTextColor(QColor("#202020"), QColor("#202020"))
        layout.addWidget(label)
        layout.addWidget(editor)
        return layout

    def get_form_data(self) -> AccountFormData:
        return AccountFormData(
            remark_name=self.remark_edit.text().strip(),
            username=self.username_edit.text().strip(),
            password=self.password_edit.text().strip(),
        )

    def validate(self) -> bool:
        data = self.get_form_data()
        if not data.remark_name or not data.username or not data.password:
            self.error_label.setText("备注名、账号、密码都得填，不然没法用。")
            self.error_label.show()
            return False

        self.error_label.hide()
        self.error_label.clear()
        return True
