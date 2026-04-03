from __future__ import annotations

from dataclasses import dataclass

from domain.models.traffic import PortalHiddenFields


@dataclass(slots=True)
class PortalPageData:
    login_url: str
    html: str
    hidden_fields: PortalHiddenFields
    cookies: dict[str, str]


@dataclass(slots=True)
class HttpResponseData:
    final_url: str
    status: int
    reason: str
    raw_body: bytes
    text: str
    cookies: dict[str, str]


@dataclass(slots=True)
class YiiLoginFormData:
    csrf_name: str
    csrf_value: str
    captcha_url: str
    action_url: str
