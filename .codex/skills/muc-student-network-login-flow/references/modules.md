# 关键模块职责

- `application/backend.rs`
处理登录、刷新、下线、日志、事件、运行时状态。

- `infrastructure/network/auth_portal_client.rs`
处理 portal 登录页、隐藏字段、HTTP 登录和 Yii 验证码登录。

- `infrastructure/network/self_service_panel_client.rs`
处理自助面板登录、访问 `/home`、在线设备、本机下线。

- `infrastructure/parsers/`
只做 HTML 和字段解析。

- `domain/policies/account_selection.rs`
找当前在线账号。

- `domain/policies/traffic_math.rs`
算配额摘要、卡片顺序、自动切号候选。
