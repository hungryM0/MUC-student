from __future__ import annotations

import base64
import http.client
import socket
import ssl
from http.cookies import SimpleCookie
from urllib.parse import urljoin, urlsplit

from infrastructure.settings import AppSettings
from infrastructure.network.models import HttpResponseData


class HttpTransport:
    def __init__(self, settings: AppSettings) -> None:
        self._settings = settings

    def request(
        self,
        method: str,
        url: str,
        headers: dict[str, str],
        body: str,
        cookies: dict[str, str],
        max_redirects: int,
    ) -> HttpResponseData:
        modes = [False]
        if self._settings.bind_preferred_source_ip and self._settings.preferred_source_ip:
            modes = [True, False]

        last_error: Exception | None = None
        for use_source_ip in modes:
            try:
                return self._request_with_mode(
                    method=method,
                    url=url,
                    headers=headers,
                    body=body,
                    cookies=cookies,
                    max_redirects=max_redirects,
                    use_source_ip=use_source_ip,
                )
            except Exception as exc:
                last_error = exc

        extra_hint = ""
        if isinstance(last_error, socket.gaierror):
            extra_hint = "DNS 解析失败（无法解析认证域名），通常是当前网络或 DNS 异常，不是账号密码错误。"

        preferred_source_ip_text = (
            self._settings.preferred_source_ip if self._settings.bind_preferred_source_ip else "disabled"
        )
        raise RuntimeError(
            f"HTTP 请求失败，interface={self._settings.preferred_interface_name}, "
            f"preferred_source_ip={preferred_source_ip_text}, "
            f"bind_enabled={self._settings.bind_preferred_source_ip}, url={url}, error={last_error}"
            + (f"，hint={extra_hint}" if extra_hint else "")
        )

    def _request_with_mode(
        self,
        method: str,
        url: str,
        headers: dict[str, str],
        body: str,
        cookies: dict[str, str],
        max_redirects: int,
        use_source_ip: bool,
    ) -> HttpResponseData:
        current_url = url
        current_method = method.upper()
        current_body = body
        cookie_jar = dict(cookies)

        for _ in range(max_redirects + 1):
            parts = urlsplit(current_url)
            if not parts.scheme or not parts.hostname:
                raise RuntimeError(f"无效 URL: {current_url}")

            path = parts.path or "/"
            if parts.query:
                path = f"{path}?{parts.query}"

            request_headers = {
                "Host": parts.netloc,
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0",
                "Connection": "close",
                **headers,
            }
            if cookie_jar:
                request_headers["Cookie"] = "; ".join(f"{k}={v}" for k, v in cookie_jar.items())

            body_bytes = current_body.encode("utf-8") if current_body else None
            connection = self._build_connection(
                scheme=parts.scheme,
                host=parts.hostname,
                port=parts.port,
                use_source_ip=use_source_ip,
            )
            try:
                connection.request(current_method, path, body=body_bytes, headers=request_headers)
                response = connection.getresponse()
                raw_body = response.read()
                self._merge_cookies(cookie_jar, response.getheaders())

                if response.status >= 400:
                    charset = response.headers.get_content_charset() or "utf-8"
                    text = raw_body.decode(charset, errors="ignore")
                    raise RuntimeError(
                        f"请求失败: status={response.status}, reason={response.reason}, "
                        f"url={current_url}, body={text[:200]}"
                    )

                if response.status in {301, 302, 303, 307, 308}:
                    location = response.getheader("Location")
                    if not location:
                        raise RuntimeError(f"重定向缺少 Location: {current_url}")
                    current_url = urljoin(current_url, location)
                    if response.status in {301, 302, 303}:
                        current_method = "GET"
                        current_body = ""
                    continue

                charset = response.headers.get_content_charset() or "utf-8"
                return HttpResponseData(
                    final_url=current_url,
                    status=response.status,
                    reason=response.reason,
                    raw_body=raw_body,
                    text=raw_body.decode(charset, errors="ignore"),
                    cookies=cookie_jar,
                )
            finally:
                connection.close()

        raise RuntimeError(f"重定向次数过多: {url}")

    @staticmethod
    def _merge_cookies(cookie_jar: dict[str, str], headers: list[tuple[str, str]]) -> None:
        for header_name, header_value in headers:
            if header_name.lower() != "set-cookie":
                continue
            simple_cookie = SimpleCookie()
            simple_cookie.load(header_value)
            for key, morsel in simple_cookie.items():
                cookie_jar[key] = morsel.value

    def _build_connection(
        self,
        scheme: str,
        host: str,
        port: int | None,
        use_source_ip: bool,
    ) -> http.client.HTTPConnection:
        source_address = (self._settings.preferred_source_ip, 0) if use_source_ip else None
        if scheme == "https":
            return http.client.HTTPSConnection(
                host=host,
                port=port or 443,
                timeout=10,
                source_address=source_address,
                context=ssl.create_default_context(),
            )
        if scheme == "http":
            return http.client.HTTPConnection(
                host=host,
                port=port or 80,
                timeout=10,
                source_address=source_address,
            )
        raise RuntimeError(f"不支持的协议: {scheme}")


def encode_password(password: str) -> str:
    encoded = base64.b64encode(password.encode("utf-8")).decode("ascii")
    return "{B}" + encoded

