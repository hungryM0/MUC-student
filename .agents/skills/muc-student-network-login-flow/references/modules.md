# 关键模块职责

- `application/backend.rs`
  编排登录、刷新、下线、自动切号、事件、运行时状态。

- `application/services/dashboard_refresh_service.rs`
  收口 dashboard 刷新流程。

- `application/services/session_service.rs`
  处理会话相关用例。

- `application/services/portal_snapshot_service.rs`
  处理成功页快照、缓存补齐和异常回退。

- `application/services/account_traffic_service.rs`
  把自助面板数据组装成账号流量快照。

- `application/services/snapshot_mapper.rs`
  组装前端快照 DTO。

- `infrastructure/network/legacy_portal_auth_client.rs`
  处理 portal 登录、覆盖切号和当前 IP 下线。

- `infrastructure/network/legacy_portal_status_client.rs`
  读取 portal 在线信息和成功页状态。

- `infrastructure/network/self_service_panel_client.rs`
  处理 SSO、自助面板 `/home` 和在线设备页面。

- `infrastructure/parsers/`
  只做 HTML 和字段解析。

- `domain/policies/traffic_math.rs`
  处理流量计算、卡片顺序、自动切号候选。
