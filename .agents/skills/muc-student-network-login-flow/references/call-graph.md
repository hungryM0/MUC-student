# 调用链索引

## 登录和切号

- `Backend::login_selected_account`
- `Backend::login_selected_account_inner`
- `NetworkStatusService::detect_network_status`
- `Backend::detect_current_online_account_fast`
- `LegacyPortalStatusClient::fetch_online_info`
- `LegacyPortalAuthClient::switch_account`
- `LegacyPortalAuthClient::verify_login`

## 刷新

- `Backend::run_refresh`
- `Backend::refresh_inner`
- `DashboardRefreshService`
- `LegacyPortalStatusClient::fetch_success_info`
- `SelfServicePanelClient::fetch_sso_html`
- `AccountTrafficService::snapshot_from_panel_home`
- `portal_snapshot_service`
- `build_status_card_order`
- `save_cached_traffic_snapshots`
- `Backend::try_auto_switch`

## 本机下线

- `Backend::logout_local_device_inner`
- `LegacyPortalAuthClient::logout_current_ip`

## 解析器

- `legacy_portal_online_info_parser`
- `legacy_portal_success_page_parser`
- `panel_home_parser`
- `online_device_parser`
- `portal_page_parser`
