from __future__ import annotations

from PySide6.QtCore import QTimer
from PySide6.QtGui import QCloseEvent, QColor
from PySide6.QtWidgets import QApplication, QStyle, QSystemTrayIcon
from qfluentwidgets.common.icon import Action, FluentIcon as FIF
from qfluentwidgets.components.dialog_box.dialog import MessageBox
from qfluentwidgets.components.navigation.navigation_panel import NavigationItemPosition
from qfluentwidgets.components.widgets.menu import SystemTrayMenu
from qfluentwidgets.window.fluent_window import FluentWindow

from app.container import AppContainer
from application.view_models import SettingsViewModel
from ui.app_text import APP_NAME
from ui.controllers.tray_controller import TrayController
from ui.dialogs.account_dialog import AccountDialog
from ui.pages.home_page import HomePage
from ui.pages.settings_page import SettingsPage
from ui.pages.status_page import StatusPage


class MainWindow(FluentWindow):
    def __init__(self, container: AppContainer) -> None:
        super().__init__()
        self._container = container
        self.controller = container.controller
        self.settings = container.settings
        self._centered_once = False
        self._tray_icon: QSystemTrayIcon | None = None
        self._tray_controller = TrayController(
            minimize_to_tray_on_close=self.controller.preferences.minimize_to_tray_on_close
        )

        self.home_page = HomePage(self)
        self.status_page = StatusPage(self)
        self.settings_page = SettingsPage(
            portal_url=self.settings.portal_url,
            traffic_portal_url=self.settings.traffic_portal_url,
            minimize_to_tray_on_close=self.controller.preferences.minimize_to_tray_on_close,
            auto_switch_account_on_traffic_exhausted=self.controller.preferences.auto_switch_account_on_traffic_exhausted,
            parent=self,
        )

        self._init_navigation()
        self._bind_signals()
        self._init_window()
        self._init_tray_icon()
        self._seed_existing_logs()
        self.controller.initialize()
        QTimer.singleShot(400, lambda: self.controller.refresh_status_page_data(force_quota_refresh=False))

    def _init_navigation(self) -> None:
        self.addSubInterface(self.home_page, FIF.HOME, "主页")
        self.addSubInterface(self.status_page, FIF.INFO, "状态")
        self.addSubInterface(self.settings_page, FIF.SETTING, "设置", NavigationItemPosition.BOTTOM)

    def _bind_signals(self) -> None:
        self.home_page.relogin_requested.connect(self.controller.start_login)
        self.home_page.selected_account_changed.connect(self.controller.select_account)
        self.status_page.add_account_requested.connect(self._show_add_account_dialog)
        self.status_page.edit_account_requested.connect(self._show_edit_account_dialog)
        self.status_page.delete_account_requested.connect(self.controller.delete_account)
        self.status_page.logout_local_requested.connect(self.controller.logout_local_device_for_account)
        self.status_page.refresh_requested.connect(lambda: self.controller.refresh_status_page_data(force_quota_refresh=True))
        self.settings_page.minimize_to_tray_changed.connect(self.controller.set_minimize_to_tray_on_close)
        self.settings_page.auto_switch_account_changed.connect(self.controller.set_auto_switch_account_on_traffic_exhausted)

        self.controller.home_changed.connect(self.home_page.apply_view_model)
        self.controller.status_changed.connect(self.status_page.apply_view_model)
        self.controller.settings_changed.connect(self._apply_settings_view_model)
        self.controller.log_arrived.connect(self.home_page.append_log)
        self.controller.warning_requested.connect(self._show_warning)

    def _init_window(self) -> None:
        self.resize(1100, 760)
        self.setWindowTitle(APP_NAME)
        self.setCustomBackgroundColor(QColor("#f3f3f3"), QColor("#f3f3f3"))
        self.navigationInterface.setAcrylicEnabled(True)
        self.navigationInterface.setExpandWidth(self._normalize_navigation_width(self.settings.navigation_expand_width))
        self.navigationInterface.setCollapsible(False)
        self.navigationInterface.setMenuButtonVisible(False)
        self.navigationInterface.expand(useAni=False)
        self.setMicaEffectEnabled(True)
        QTimer.singleShot(0, self._center_on_screen)

    def _center_on_screen(self) -> None:
        if self._centered_once:
            return
        screen = self.screen()
        if screen is None:
            QTimer.singleShot(50, self._center_on_screen)
            return
        frame = self.frameGeometry()
        frame.moveCenter(screen.availableGeometry().center())
        self.move(frame.topLeft())
        self._centered_once = True

    def _seed_existing_logs(self) -> None:
        for line in self.controller.get_existing_log_lines():
            self.home_page.append_log(line)

    def _init_tray_icon(self) -> None:
        if not QSystemTrayIcon.isSystemTrayAvailable():
            self._tray_icon = None
            return
        tray_icon = QSystemTrayIcon(self)
        icon = self.windowIcon()
        if icon.isNull():
            icon = self.style().standardIcon(QStyle.StandardPixmap.SP_ComputerIcon)
        tray_icon.setIcon(icon)
        tray_icon.setToolTip(APP_NAME)
        tray_icon.activated.connect(self._on_tray_icon_activated)

        tray_menu = SystemTrayMenu(parent=self)
        tray_menu.addActions(
            [
                Action(FIF.HOME, "显示主窗口", triggered=self._restore_window_from_tray),
                Action(FIF.CLOSE, "退出程序", triggered=self._exit_app_from_tray),
            ]
        )
        tray_icon.setContextMenu(tray_menu)
        tray_icon.show()
        self._tray_icon = tray_icon

    def _on_tray_icon_activated(self, reason: QSystemTrayIcon.ActivationReason) -> None:
        if reason in {QSystemTrayIcon.ActivationReason.Trigger, QSystemTrayIcon.ActivationReason.DoubleClick}:
            self._restore_window_from_tray()

    def _restore_window_from_tray(self) -> None:
        if self.isMinimized():
            self.showNormal()
        else:
            self.show()
        self.raise_()
        self.activateWindow()

    def _exit_app_from_tray(self) -> None:
        self._tray_controller.mark_quitting_from_tray()
        self.close()
        app = QApplication.instance()
        if app is not None:
            QTimer.singleShot(0, app.quit)

    def _show_add_account_dialog(self) -> None:
        dialog = AccountDialog("添加账号", parent=self)
        if dialog.exec() == 0:
            return
        form_data = dialog.get_form_data()
        self.controller.add_account(
            remark_name=form_data.remark_name,
            username=form_data.username,
            password=form_data.password,
        )

    def _show_edit_account_dialog(self, account_id: str) -> None:
        account = self.controller.get_account(account_id)
        if account is None:
            self._show_warning("编辑账号失败", "找不到这个账号，可能已经被删了")
            return
        dialog = AccountDialog("编辑账号", account=account, parent=self)
        if dialog.exec() == 0:
            return
        form_data = dialog.get_form_data()
        self.controller.edit_account(
            account_id=account_id,
            remark_name=form_data.remark_name,
            username=form_data.username,
            password=form_data.password,
        )

    def _apply_settings_view_model(self, view_model: SettingsViewModel) -> None:
        self._tray_controller.set_minimize_to_tray_on_close(view_model.minimize_to_tray_on_close)
        self.settings_page.apply_view_model(view_model)

    def _show_warning(self, title: str, content: str) -> None:
        message_box = MessageBox(title, content, self)
        message_box.yesButton.setText("知道了")
        message_box.hideCancelButton()
        message_box.exec()

    def closeEvent(self, event: QCloseEvent) -> None:
        tray_ready = self._tray_icon is not None and self._tray_icon.isVisible()
        if self._tray_controller.minimize_to_tray_on_close and tray_ready and not self._tray_controller.quitting_from_tray:
            event.ignore()
            self.hide()
            if self._tray_icon is not None and not self._tray_controller.tray_hint_shown:
                self._tray_icon.showMessage(
                    APP_NAME,
                    "程序已最小化到托盘，可在托盘菜单里恢复或退出。",
                    QSystemTrayIcon.MessageIcon.Information,
                    3000,
                )
                self._tray_controller.mark_tray_hint_shown()
            return

        self.controller.shutdown()
        if self._tray_icon is not None:
            self._tray_icon.hide()
        event.accept()
        super().closeEvent(event)
        if self._tray_controller.quitting_from_tray:
            app = QApplication.instance()
            if app is not None:
                QTimer.singleShot(0, app.quit)

    @staticmethod
    def _normalize_navigation_width(width: int) -> int:
        return max(43, min(600, int(width)))

