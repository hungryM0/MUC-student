梳理或修改 MUC-student 的校园网登录、切号、状态刷新、SSO 面板、流量查询、在线设备和本机下线链路。

先读 `CLAUDE.md`，再从 application 入口往下查，不要先改 parser 或 HTTP client。

## 入口

- 登录和切号：`Backend::login_selected_account`、`login_selected_account_inner`
- 刷新状态：`Backend::run_refresh`、`refresh_inner`
- 本机下线：`Backend::logout_local_device_inner`
- 自动切号：`Backend::try_auto_switch`

## 主链路

**登录和切号**

- 无登录态时，直接走 portal HTTP 登录。
- 已登录时，切号走登录页表单覆盖登录（`srun_portal_pc.php?ac_id=1&`）。
- `/include/auth_action.php` 可返回 `IP has been online, please logout.`，不能当切号主链路。
- 不要做"先本机下线再登录"。这是错路。

**刷新、流量和在线设备**

- 先查本机 IP。
- 读成功页 `srun_portal_pc_success.php`，解析本机 IP、上网用户、已用流量、计费方式。
- SSO URL 是 `traffic_portal_url` 的 origin 加 `/site/sso?data=base64(username:username)`。
- SSO 第一跳 302 设置 `PHPSESSID_8800`，必须保留 cookie 后再请求 `/home`。
- `/home` 解析产品、套餐、已用流量、在线设备、下线链接。
- 刷新只查当前本机在线账号，不扫描所有账号。

**本机下线**

- 只用于用户主动下号。
- 必须先有有效本机 IP 和 `current_online_account_id`。
- 不用于切号前置步骤。

## 分层落点

- 流程编排：`src-core/src/application/backend.rs` 和 `src-core/src/application/services/`
- portal 登录、状态页、SSO 请求：`src-core/src/infrastructure/network/`
- HTML 提取：`src-core/src/infrastructure/parsers/`
- 自动切号和流量排序：`src-core/src/domain/policies/`
- DTO：`src-core/src/application/dto.rs`

parser 只解析。HTTP client 只访问外部系统。账号选择和流程判断不要塞进去。

## 改动后检查

改登录、刷新、下线或自动切号后，同时确认：日志是否还能说明当前步骤，`app_state` 是否正确写回，`AppSnapshotDto` 是否仍兼容前端，缓存是否正确读写，事件是否仍能驱动界面刷新。
