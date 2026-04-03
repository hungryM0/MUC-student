from __future__ import annotations

import argparse
import sys
from dataclasses import replace
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[1]
if str(ROOT_DIR) not in sys.path:
    sys.path.insert(0, str(ROOT_DIR))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="最小 HTTP 登录验证器")
    parser.add_argument("--portal-url", default="", help="认证页地址，默认使用应用配置")
    parser.add_argument("--username", default="", help="账号，不传就使用当前选中的账号")
    parser.add_argument("--password", default="", help="密码，不传就使用当前选中的账号")
    parser.add_argument(
        "--bind-source-ip",
        action="store_true",
        help="强制尝试绑定 preferred_source_ip，失败后仍会自动降级直连",
    )
    return parser


def resolve_account(args: argparse.Namespace):
    from domain.models.account import PortalAccount
    from infrastructure.persistence.account_store_repository import AccountStoreRepository
    from infrastructure.persistence.runtime_paths_provider import RuntimePathsProvider

    username = args.username.strip()
    password = args.password.strip()
    if username or password:
        if not username or not password:
            raise RuntimeError("命令行账号和密码必须一起传，别只给一半")
        return PortalAccount(
            id="cli-temporary-account",
            remark_name="命令行临时账号",
            username=username,
            password=password,
        )

    account_repo = AccountStoreRepository(RuntimePathsProvider())
    store = account_repo.load_store()
    account = account_repo.get_selected_account(store)
    if account is None:
        raise RuntimeError("当前没有可用账号，请先在程序里添加账号")
    return account


def main() -> int:
    from infrastructure.settings import AppSettings
    from infrastructure.network.auth_portal_client import AuthPortalClient
    from infrastructure.network.http_transport import HttpTransport
    from infrastructure.captcha_ocr_gateway import CaptchaOcrGateway

    args = build_parser().parse_args()
    settings = AppSettings.load()
    settings = replace(
        settings,
        portal_url=args.portal_url or settings.portal_url,
        bind_preferred_source_ip=args.bind_source_ip or settings.bind_preferred_source_ip,
    )
    account = resolve_account(args)

    service = AuthPortalClient(settings, HttpTransport(settings), CaptchaOcrGateway())
    page_data = service.fetch_login_page()
    result = service.verify_login(account, page_data)

    print(f"登录页地址: {page_data.login_url}")
    print(f"使用账号: {account.display_name}")
    print(
        "隐藏字段: "
        f"ac_id={page_data.hidden_fields.ac_id}, "
        f"user_ip={page_data.hidden_fields.user_ip}, "
        f"nas_ip={page_data.hidden_fields.nas_ip}, "
        f"user_mac={page_data.hidden_fields.user_mac}"
    )
    print(f"验证结果: {'成功' if result.success else '失败'}")
    print(f"结果说明: {result.message}")
    print(f"接口原始返回: {result.response_text}")
    return 0 if result.success else 1


if __name__ == "__main__":
    raise SystemExit(main())

