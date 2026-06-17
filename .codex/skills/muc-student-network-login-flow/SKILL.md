---
name: muc-student-network-login-flow
description: 梳理或修改 MUC-student 的校园网认证、轻量 portal 切号、SSO 自助面板、流量查询、本机下线、在线设备识别和自动切号链路。仅在改 `src-tauri/src/application/backend.rs`、`application/services/*traffic*`、`application/services/portal_snapshot_service.rs`、`infrastructure/network/*portal*`、`self_service_panel_client.rs` 或相关 parser/策略时使用；普通前端改动不触发。
---

# MUC Student Network Login Flow

先找入口。不要先改 parser，也不要先改某个 HTTP client。

## 入口顺序

先从 `src-tauri/src/application/backend.rs` 开始读，再沿着这条线往下：

- `login_selected_account` / `login_selected_account_inner`
- `refresh_dashboard` / `run_refresh` / `refresh_inner`
- `logout_local_device` / `logout_local_device_inner`
- `try_auto_switch`

再看它们调到哪里：

- `LegacyPortalAuthClient`
- `LegacyPortalStatusClient`
- `SelfServicePanelClient`
- `AccountTrafficService`
- `PortalSnapshotService`
- `NetworkStatusService`
- `traffic_math`
- 各 parser

## 读代码顺序

1. 先确认改的是哪条链路。
2. 找 application 层入口。
3. 记下 DTO、日志、运行时状态会不会受影响。
4. 再往下读 infrastructure client 和 parser。
5. 最后才改策略和计算逻辑。

## 当前关键链路

### 登录

- 入口在 `Backend::login_selected_account`
- 实际流程在 `login_selected_account_inner`
- 会先探测本机网络状态
- 可能先从在线列表里找当前本机 IP 所属账号
- 如有必要用 `LegacyPortalAuthClient::switch_account` 轻量切号
- 再调用 `LegacyPortalAuthClient::verify_login`
- 最后写回 `app_state`、选中账号、日志和事件

### 刷新状态

- 入口在 `run_refresh` / `refresh_inner`
- 先查本机 IP
- 有 portal 登录态时优先走 `PortalSnapshotService` 轻量切号查询
- 无 portal 登录态时走 `AccountTrafficService` 通过自助面板 SSO 查询
- 用 `build_status_card_order`、`save_cached_traffic_snapshots` 写回缓存
- 刷新后可能触发 `try_auto_switch`

### 本机下线

- 入口在 `logout_local_device_inner`
- 必须先有有效本机 IP
- 必须先有 `current_online_account_id`
- 实际 portal 下线由 `LegacyPortalAuthClient::logout_current_ip` 完成

## 硬规则

- 不要跳过 application 层，直接让前端猜 infrastructure 细节。
- 不要把 parser 当业务层。parser 只做解析，不做流程判断。
- 自动切号规则放 `domain/policies`，不要塞进 HTTP client。
- 修改登录或刷新行为时，同时检查日志、状态事件、缓存写回。

## 常见坑

- 只改 `LegacyPortalAuthClient`，忘了 `PortalSnapshotService` 的并发探测和串行回退。
- 只改 response 解析，忘了 `already_online` 和预下线分支。
- 只改后端，不看前端是否依赖 `AppSnapshotDto` 字段。
- 在 infrastructure 层写选择账号逻辑，代码会发臭。
- 又把 OCR 或验证码登录链路加回来，直接判定越界。

## 参考

- 调用链索引：`references/call-graph.md`
- 关键模块职责：`references/modules.md`
