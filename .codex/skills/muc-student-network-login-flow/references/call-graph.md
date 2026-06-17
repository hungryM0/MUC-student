# 调用链索引

## 登录

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
- `LegacyPortalStatusClient::fetch_success_info`
- `PortalSnapshotService::fetch_balances_with_probe`
- `PortalSnapshotService::probe_balances_parallel`
- `PortalSnapshotService::fetch_balances_serial`
- `AccountTrafficService::fetch_balances`
- `AccountTrafficService::to_snapshot_map`
- `build_status_card_order`
- `save_cached_traffic_snapshots`
- `Backend::try_auto_switch`

## 本机下线

- `Backend::logout_local_device_inner`
- `LegacyPortalAuthClient::logout_current_ip`
- `LegacyPortalAuthClient::logout_with_success_page`
