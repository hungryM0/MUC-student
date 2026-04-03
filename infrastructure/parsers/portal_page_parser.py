from __future__ import annotations

import re
from html import unescape as html_unescape
from html.parser import HTMLParser
from urllib.parse import parse_qs, urljoin, urlsplit

from domain.models.traffic import PortalHiddenFields
from infrastructure.network.models import YiiLoginFormData


class _HiddenInputParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.values: dict[str, str] = {
            "ac_id": "",
            "user_ip": "",
            "nas_ip": "",
            "user_mac": "",
        }

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag.lower() != "input":
            return
        attr_map = {key.lower(): (value or "") for key, value in attrs}
        name = attr_map.get("name", "")
        if name in self.values:
            self.values[name] = attr_map.get("value", "")


class _YiiLoginFormParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self._form_depth = 0
        self._inside_login_form = False
        self.action_url = "/login"
        self.csrf_name = ""
        self.csrf_value = ""
        self.captcha_url = ""

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attr_map = {key.lower(): (value or "") for key, value in attrs}
        lower_tag = tag.lower()

        if lower_tag == "form":
            self._form_depth += 1
            form_id = attr_map.get("id", "")
            action = attr_map.get("action", "")
            if form_id == "login-form" or "/login" in action:
                self._inside_login_form = True
                self.action_url = action or "/login"
            return

        if lower_tag == "input" and self._inside_login_form:
            name = attr_map.get("name", "")
            if name.startswith("_csrf"):
                self.csrf_name = name
                self.csrf_value = attr_map.get("value", "")
            return

        if lower_tag == "img":
            element_id = attr_map.get("id", "")
            src = attr_map.get("src", "")
            if element_id == "loginform-verifycode-image" or "captcha" in src:
                self.captcha_url = src

    def handle_endtag(self, tag: str) -> None:
        if tag.lower() != "form":
            return
        if self._form_depth > 0:
            self._form_depth -= 1
        if self._inside_login_form and self._form_depth == 0:
            self._inside_login_form = False


def is_yii_login_page(html: str) -> bool:
    lower_html = html.lower()
    return (
        "loginform[username]" in lower_html
        and "loginform[password]" in lower_html
        and "loginform[verifycode]" in lower_html
    )


def is_traffic_home_page(html: str) -> bool:
    return "产品名称" in html and "已用流量" in html and "产品余额" in html


def parse_hidden_fields(html: str, page_url: str) -> PortalHiddenFields:
    parser = _HiddenInputParser()
    parser.feed(html)
    parser.close()

    values = parser.values
    if not values["ac_id"]:
        query = parse_qs(urlsplit(page_url).query, keep_blank_values=True)
        values["ac_id"] = query.get("ac_id", [""])[0]

    return PortalHiddenFields(
        ac_id=values["ac_id"],
        user_ip=values["user_ip"],
        nas_ip=values["nas_ip"],
        user_mac=values["user_mac"],
    )


def parse_yii_login_form(html: str, page_url: str) -> YiiLoginFormData:
    parser = _YiiLoginFormParser()
    parser.feed(html)
    parser.close()

    if not parser.csrf_name or not parser.csrf_value:
        raise RuntimeError("登录页缺少 CSRF 字段，无法提交表单")

    captcha_src = parser.captcha_url or "/site/captcha?refresh=1"
    return YiiLoginFormData(
        csrf_name=parser.csrf_name,
        csrf_value=parser.csrf_value,
        captcha_url=urljoin(page_url, captcha_src),
        action_url=urljoin(page_url, parser.action_url or "/login"),
    )


def extract_yii_error_message(html: str) -> str:
    text = re.sub(r"<[^>]+>", " ", html)
    text = re.sub(r"\s+", " ", text).strip()
    for key in [
        "验证码不正确",
        "用户名不能为空",
        "密码不能为空",
        "用户名或密码错误",
        "请修复以下错误",
    ]:
        if key in text:
            return key
    return text[:80] if text else "登录失败"


def normalize_captcha_code(raw_code: str | None) -> str:
    raw_text = raw_code or ""
    normalized = re.sub(r"[^0-9A-Za-z]", "", raw_text.strip())
    return normalized[:4]


def extract_meta_content(html: str, meta_name: str) -> str:
    patterns = [
        rf'<meta[^>]*name=["\']{re.escape(meta_name)}["\'][^>]*content=["\']([^"\']+)["\']',
        rf'<meta[^>]*content=["\']([^"\']+)["\'][^>]*name=["\']{re.escape(meta_name)}["\']',
    ]
    for pattern in patterns:
        match = re.search(pattern, html or "", flags=re.IGNORECASE)
        if match:
            return html_unescape(match.group(1)).strip()
    return ""
