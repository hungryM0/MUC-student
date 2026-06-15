# 调用链索引

## 登录

- `Backend::login_selected_account`
- `Backend::login_selected_account_inner`
- `NetworkStatusService::detect_network_status`
- `AccountTrafficService::fetch_balances`
- `find_current_online_account`
- `SelfServicePanelClient::logout_local_device`
- `AuthPortalClient::verify_login`

## 刷新

- `Backend::run_refresh`
- `Backend::refresh_inner`
- `AccountTrafficService::fetch_balances`
- `AccountTrafficService::to_snapshot_map`
- `build_status_card_order`
- `save_cached_traffic_snapshots`
- `Backend::try_auto_switch`

## 本机下线

- `Backend::logout_local_device_inner`
- `SelfServicePanelClient::logout_local_device`
- `parse_online_devices`
- `extract_csrf_meta`
