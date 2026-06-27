---
name: muc-student-network-login-flow
description: 梳理或修改 MUC-student 的校园网登录、覆盖切号、成功页状态、SSO 自助面板、流量查询、在线设备、本机下线和自动切号链路。仅在改 `src-core/src/application/backend.rs`、`src-core/src/application/services/*traffic*`、`src-core/src/application/services/*snapshot*`、`src-core/src/infrastructure/network/*portal*`、`src-core/src/infrastructure/network/self_service_panel_client.rs`、相关 parser 或自动切号策略时使用；普通前端和样式改动不触发。
---

# MUC Student Network Login Flow

先读 `AGENTS.md`，再从 application 入口往下查。不要先改 parser 或 HTTP client。

## 入口

- 登录和切号：`Backend::login_selected_account`、`login_selected_account_inner`。
- 刷新状态：`Backend::run_refresh`、`refresh_inner`。
- 本机下线：`Backend::logout_local_device_inner`。
- 自动切号：`Backend::try_auto_switch`。

## 当前主链路

### 登录和切号

- 无登录态时，直接走 portal HTTP 登录。
- 已登录时，切号走登录页表单覆盖登录。
- 覆盖登录 URL：`http://rz.muc.edu.cn/srun_portal_pc.php?ac_id=1&`。
- 表单字段包含 `action=login`、`username`、明文 `password`、`ac_id=1`、`save_me=0`、`drop=0`、`ajax=1`。
- `/include/auth_action.php` 可返回 `IP has been online, please logout.`，不要把它当切号主链路。
- 不要做“先本机下线再登录”。这是错路。

### 刷新、流量和在线设备

- 先查本机 IP。
- 读成功页 `srun_portal_pc_success.php`。
- 从成功页解析本机 IP、上网用户、已用流量、计费方式。
- 用成功页账号匹配本地账号。
- SSO URL 是 `traffic_portal_url` 的 origin 加 `/site/sso?data=base64(username:username)`。
- SSO 第一跳 302 设置 `PHPSESSID_8800`，必须保留 cookie 后再请求 `/home`。
- `/home` 解析产品、套餐、计费策略、已用流量、在线设备、下线链接。
- SSO 或 `/home` 失败时，退回成功页快照和缓存补齐。
- 刷新只查当前本机在线账号，不要扫描所有账号。

### 本机下线

- 只用于用户主动下号。
- 必须先有有效本机 IP。
- 必须先有 `current_online_account_id`。
- portal 下线由 `LegacyPortalAuthClient::logout_current_ip` 处理。
- 不用于切号前置步骤。

## 分层落点

- 流程编排：`src-core/src/application/backend.rs` 和 `src-core/src/application/services/`。
- portal 登录、状态页、SSO 请求：`src-core/src/infrastructure/network/`。
- HTML 提取：`src-core/src/infrastructure/parsers/`。
- 自动切号和流量排序：`src-core/src/domain/policies/traffic_math.rs`。
- DTO：`src-core/src/application/dto.rs`。

parser 只解析。HTTP client 只访问外部系统。账号选择和流程判断不要塞进去。

## 改动检查

改登录、刷新、下线或自动切号时，同时检查：

- 日志是否还能说明当前步骤。
- `app_state` 是否正确写回。
- `AppSnapshotDto` 是否仍兼容前端。
- 缓存是否正确读写。
- 事件是否仍能驱动界面刷新。

## 参考

- 调用链索引：`references/call-graph.md`
- 关键模块职责：`references/modules.md`
