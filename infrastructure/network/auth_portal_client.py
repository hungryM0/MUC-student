from __future__ import annotations

from urllib.parse import urlencode, urljoin, urlsplit

from domain.models.account import PortalAccount
from domain.models.traffic import LoginResult, PortalHiddenFields
from infrastructure.settings import AppSettings
from infrastructure.network.http_transport import HttpTransport, encode_password
from infrastructure.network.models import HttpResponseData, PortalPageData
from infrastructure.captcha_ocr_gateway import CaptchaOcrGateway
from infrastructure.parsers.portal_page_parser import (
    extract_yii_error_message,
    is_yii_login_page,
    normalize_captcha_code,
    parse_hidden_fields,
    parse_yii_login_form,
)


class AuthPortalClient:
    _RESPONSE_IP_ALREADY_ONLINE = "IP has been online, please logout."

    def __init__(self, settings: AppSettings, transport: HttpTransport, ocr_gateway: CaptchaOcrGateway) -> None:
        self._settings = settings
        self._transport = transport
        self._ocr_gateway = ocr_gateway

    def fetch_login_page(self) -> PortalPageData:
        response = self._transport.request(
            method="GET",
            url=self._settings.portal_url,
            headers={},
            body="",
            cookies={},
            max_redirects=5,
        )
        return PortalPageData(
            login_url=response.final_url,
            html=response.text,
            hidden_fields=parse_hidden_fields(response.text, response.final_url),
            cookies=response.cookies,
        )

    def verify_login(self, account: PortalAccount, page_data: PortalPageData | None = None) -> LoginResult:
        page_data = page_data or self.fetch_login_page()
        if is_yii_login_page(page_data.html):
            return self._verify_login_yii(account)

        if not page_data.hidden_fields.ac_id:
            raise RuntimeError("登录页缺少 ac_id，无法继续验证")

        post_url = urljoin(page_data.login_url, "/include/auth_action.php")
        payload = urlencode(
            [
                ("action", "login"),
                ("username", account.username),
                ("password", encode_password(account.password)),
                ("ac_id", page_data.hidden_fields.ac_id),
                ("user_ip", page_data.hidden_fields.user_ip),
                ("nas_ip", page_data.hidden_fields.nas_ip),
                ("user_mac", page_data.hidden_fields.user_mac),
                ("save_me", "0"),
                ("ajax", "1"),
            ]
        )
        response = self._transport.request(
            method="POST",
            url=post_url,
            headers=self._build_login_headers(page_data.login_url),
            body=payload,
            cookies=page_data.cookies,
            max_redirects=1,
        )

        response_text = response.text.strip()
        already_online = response_text == self._RESPONSE_IP_ALREADY_ONLINE
        success = response_text.startswith("login_ok,")
        if already_online:
            message = "当前 IP 已在线，无法确认是否为目标账号"
        elif success:
            message = "HTTP 接口登录成功"
        else:
            message = f"HTTP 接口登录失败：{response_text or '服务器未返回内容'}"
        return LoginResult(
            success=success,
            message=message,
            login_url=page_data.login_url,
            hidden_fields=page_data.hidden_fields,
            response_text=response_text,
            checked_at=time_now(),
            already_online=already_online,
        )

    def _verify_login_yii(self, account: PortalAccount) -> LoginResult:
        try:
            response = self._login_yii_with_ocr(account)
        except Exception as exc:
            return LoginResult(
                success=False,
                message=f"HTTP 表单登录失败：{exc}",
                login_url=self._settings.portal_url,
                hidden_fields=PortalHiddenFields(ac_id="", user_ip="", nas_ip="", user_mac=""),
                response_text=str(exc),
                checked_at=time_now(),
            )

        return LoginResult(
            success=True,
            message="HTTP 表单登录成功（OCR 验证码）",
            login_url=response.final_url,
            hidden_fields=PortalHiddenFields(ac_id="", user_ip="", nas_ip="", user_mac=""),
            response_text=response.final_url,
            checked_at=time_now(),
            already_online=False,
        )

    def _login_yii_with_ocr(self, account: PortalAccount, max_attempts: int = 10) -> HttpResponseData:
        self._ocr_gateway.ensure_ready()
        last_error = "未知错误"

        for _ in range(max_attempts):
            page_response = self._transport.request(
                method="GET",
                url=self._settings.portal_url,
                headers={},
                body="",
                cookies={},
                max_redirects=5,
            )
            if not is_yii_login_page(page_response.text):
                raise RuntimeError("入口页不是可识别的登录表单")

            form = parse_yii_login_form(page_response.text, page_response.final_url)
            captcha_response = self._transport.request(
                method="GET",
                url=form.captcha_url,
                headers={"Referer": page_response.final_url},
                body="",
                cookies=page_response.cookies,
                max_redirects=2,
            )
            captcha_code = normalize_captcha_code(self._ocr_gateway.classification(captcha_response.raw_body))
            if len(captcha_code) < 3:
                last_error = f"OCR 识别结果无效：{captcha_code or '<empty>'}"
                continue

            payload = urlencode(
                [
                    (form.csrf_name, form.csrf_value),
                    ("LoginForm[username]", account.username),
                    ("LoginForm[password]", account.password),
                    ("LoginForm[verifyCode]", captcha_code),
                ]
            )
            response = self._transport.request(
                method="POST",
                url=form.action_url,
                headers=self._build_form_headers(page_response.final_url),
                body=payload,
                cookies=page_response.cookies,
                max_redirects=3,
            )

            if not is_yii_login_page(response.text):
                return response

            error_text = extract_yii_error_message(response.text)
            if "验证码不正确" in error_text:
                last_error = f"验证码识别失败（OCR={captcha_code}）"
                continue
            if "用户名" in error_text or "密码" in error_text:
                raise RuntimeError(error_text)
            last_error = error_text or "表单提交后仍停留登录页"

        raise RuntimeError(f"验证码连续识别失败，已重试 {max_attempts} 次，最后错误：{last_error}")

    @staticmethod
    def _build_login_headers(referer_url: str) -> dict[str, str]:
        parts = urlsplit(referer_url)
        origin = f"{parts.scheme}://{parts.netloc}"
        return {
            "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
            "Origin": origin,
            "Referer": referer_url,
            "X-Requested-With": "XMLHttpRequest",
        }

    @staticmethod
    def _build_form_headers(referer_url: str) -> dict[str, str]:
        parts = urlsplit(referer_url)
        origin = f"{parts.scheme}://{parts.netloc}"
        return {
            "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
            "Origin": origin,
            "Referer": referer_url,
        }


def time_now():
    from datetime import datetime
    return datetime.now()

