from __future__ import annotations

import time
from urllib.parse import urlencode, urljoin, urlsplit

from domain.models.account import PortalAccount
from infrastructure.settings import AppSettings
from infrastructure.network.http_transport import HttpTransport
from infrastructure.network.models import HttpResponseData
from infrastructure.captcha_ocr_gateway import CaptchaOcrGateway
from infrastructure.parsers.online_device_parser import parse_online_devices
from infrastructure.parsers.panel_home_parser import extract_csrf_meta
from infrastructure.parsers.portal_page_parser import (
    extract_yii_error_message,
    is_traffic_home_page,
    is_yii_login_page,
    normalize_captcha_code,
    parse_yii_login_form,
)


class SelfServicePanelClient:
    _LOCAL_DEVICE_VERIFY_RETRY_DELAYS_SECONDS: tuple[float, ...] = (0.0, 0.6, 1.2)

    def __init__(self, settings: AppSettings, transport: HttpTransport, ocr_gateway: CaptchaOcrGateway) -> None:
        self._settings = settings
        self._transport = transport
        self._ocr_gateway = ocr_gateway

    def fetch_authenticated_html(self, account: PortalAccount, path: str = "/home") -> str:
        return self.fetch_authenticated_page(account, path).text

    def fetch_authenticated_page(self, account: PortalAccount, path: str = "/home") -> HttpResponseData:
        target_url = self._settings.traffic_portal_url.strip() or self._settings.portal_url
        page_response = self._transport.request(
            method="GET",
            url=target_url,
            headers={},
            body="",
            cookies={},
            max_redirects=5,
        )
        target_path = path.strip() or "/home"

        if is_traffic_home_page(page_response.text):
            return page_response

        if is_yii_login_page(page_response.text):
            response = self._login_yii_with_ocr(account, page_response)
            if urlsplit(response.final_url).path.rstrip("/") != target_path.rstrip("/"):
                response = self._transport.request(
                    method="GET",
                    url=urljoin(response.final_url, target_path),
                    headers={},
                    body="",
                    cookies=response.cookies,
                    max_redirects=5,
                )
                if is_yii_login_page(response.text):
                    raise RuntimeError("登录态失效，访问目标页面时被重定向回登录页")
                if not is_traffic_home_page(response.text):
                    raise RuntimeError(f"访问 {target_path} 成功，但页面里没有流量表格")
            return response

        retry_response = self._transport.request(
            method="GET",
            url=urljoin(page_response.final_url, target_path),
            headers={},
            body="",
            cookies=page_response.cookies,
            max_redirects=5,
        )
        if is_traffic_home_page(retry_response.text):
            return retry_response
        if is_yii_login_page(retry_response.text):
            return self._login_yii_with_ocr(account, retry_response)

        raise RuntimeError(
            f"流量入口不匹配：query_url={self._settings.traffic_portal_url}, final_url={page_response.final_url}"
        )

    def logout_local_device(self, account: PortalAccount, local_ip: str) -> str:
        local_ip_text = (local_ip or "").strip()
        if not local_ip_text or local_ip_text == "unknown":
            raise RuntimeError("本机 IP 未知，无法执行本机下线")

        home_response = self.fetch_authenticated_page(account, "/home")
        home_url = home_response.final_url
        csrf_param, csrf_token = extract_csrf_meta(home_response.text)
        if not csrf_param or not csrf_token:
            raise RuntimeError("当前账号 /home 页面缺少 CSRF 字段，无法执行本机下线")

        local_device = self.extract_local_online_device(home_response.text, local_ip_text)
        if local_device is None:
            raise RuntimeError(f"当前账号在线信息里没找到本机 IP：{local_ip_text}")

        payload = urlencode([(csrf_param, csrf_token)])
        logout_response = self._transport.request(
            method="POST",
            url=urljoin(home_response.final_url, local_device.logout_path),
            headers=self._build_form_headers(home_url),
            body=payload,
            cookies=home_response.cookies,
            max_redirects=3,
        )

        verify_url = urljoin(home_url, "/home")
        verify_cookies = dict(logout_response.cookies)
        for delay in self._LOCAL_DEVICE_VERIFY_RETRY_DELAYS_SECONDS:
            if delay > 0:
                time.sleep(delay)
            verify_response = self._transport.request(
                method="GET",
                url=verify_url,
                headers={},
                body="",
                cookies=verify_cookies,
                max_redirects=5,
            )
            verify_cookies = dict(verify_response.cookies)
            if self.extract_local_online_device(verify_response.text, local_ip_text) is None:
                return f"本机设备下线成功：账号={account.display_name}, ip={local_ip_text}"

        raise RuntimeError("本机设备下线后校验失败：在线列表里仍然存在本机 IP 记录")

    @staticmethod
    def extract_local_online_device(html: str, local_ip: str):
        for row in parse_online_devices(html):
            if row.ip == local_ip:
                return row
        return None

    def _login_yii_with_ocr(self, account: PortalAccount, page_response: HttpResponseData, max_attempts: int = 10) -> HttpResponseData:
        self._ocr_gateway.ensure_ready()
        last_error = "未知错误"
        current_page = page_response
        for _ in range(max_attempts):
            if not is_yii_login_page(current_page.text):
                raise RuntimeError("入口页不是可识别的登录表单")

            form = parse_yii_login_form(current_page.text, current_page.final_url)
            captcha_response = self._transport.request(
                method="GET",
                url=form.captcha_url,
                headers={"Referer": current_page.final_url},
                body="",
                cookies=current_page.cookies,
                max_redirects=2,
            )
            captcha_code = normalize_captcha_code(self._ocr_gateway.classification(captcha_response.raw_body))
            if len(captcha_code) < 3:
                last_error = f"OCR 识别结果无效：{captcha_code or '<empty>'}"
                current_page = self._transport.request(
                    method="GET",
                    url=self._settings.traffic_portal_url,
                    headers={},
                    body="",
                    cookies={},
                    max_redirects=5,
                )
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
                headers=self._build_form_headers(current_page.final_url),
                body=payload,
                cookies=current_page.cookies,
                max_redirects=3,
            )

            if not is_yii_login_page(response.text):
                return response

            error_text = extract_yii_error_message(response.text)
            if "验证码不正确" in error_text:
                last_error = f"验证码识别失败（OCR={captcha_code}）"
                current_page = response
                continue
            if "用户名" in error_text or "密码" in error_text:
                raise RuntimeError(error_text)
            last_error = error_text or "表单提交后仍停留登录页"
            current_page = response

        raise RuntimeError(f"验证码连续识别失败，已重试 {max_attempts} 次，最后错误：{last_error}")

    @staticmethod
    def _build_form_headers(referer_url: str) -> dict[str, str]:
        parts = urlsplit(referer_url)
        origin = f"{parts.scheme}://{parts.netloc}"
        return {
            "Content-Type": "application/x-www-form-urlencoded; charset=UTF-8",
            "Origin": origin,
            "Referer": referer_url,
        }

