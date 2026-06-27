# 关键模块职责

- `application/backend.rs`
处理登录、刷新、下线、事件、运行时状态编排。

- `application/services/portal_snapshot_service.rs`
处理 portal 登录态下的并发探测、串行回退、恢复原账号和成功页快照。

- `infrastructure/network/legacy_portal_auth_client.rs`
处理 legacy portal 轻量登录、切号和当前 IP 下线。

- `infrastructure/network/legacy_portal_status_client.rs`
读取 legacy portal 在线信息和成功页状态。

- `infrastructure/network/self_service_panel_client.rs`
处理自助面板 SSO、访问 `/home`、在线设备页面。

- `infrastructure/parsers/`
只做 HTML 和字段解析。

- `domain/policies/traffic_math.rs`
算配额摘要、卡片顺序、自动切号候选。
