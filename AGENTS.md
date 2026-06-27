# MUC-student 协作说明

这个仓库是 MUC 校园网多账号桌面工具。

项目已进入稳定期。默认做小步、低风险改动。先读现有实现，再判断落点。不要重写已经稳定的链路。

## 当前架构

- 桌面壳：Tauri v2。
- 前端：React。
- 后端核心：Rust，放在 `src-core/`。
- 目标平台：Windows。
- CI：`.github/workflows/ci.yml`，按 Windows 约束跑。

登录态规则要记住：

- 无登录态时，可以直接走 portal HTTP 登录。
- 已经登录时，切号走登录页表单覆盖登录。
- 查流量、在线设备、本机下线，优先走成功页和 SSO 自助面板链路。
- 不要把切号改成“先下线再登录”。这是错路。

## 目录边界

- `src/`：React 前端。
- `src/pages/`：页面。
- `src/components/`：组件。
- `src/hooks/`：前端 hook。
- `src/lib/`：Tauri invoke 封装、前端 DTO、窗口工具。
- `src-core/`：可复用业务核心。
- `src-core/src/application/`：用例、服务、DTO、运行时编排。
- `src-core/src/domain/`：领域模型和纯策略。
- `src-core/src/infrastructure/`：网络、解析、持久化、凭据、安全、系统适配。
- `src-tauri/`：Tauri 桌面壳。
- `src-tauri/src/lib.rs`：command、插件注册、核心装配。
- `src-tauri/src/platform.rs`：Windows 运行路径和 HKCU Run 自启。
- `src-tauri/src/plugins/`：Tauri 插件封装。

生成物不要手改：

- `build/`
- `dist/`
- `target/`
- `src-tauri/target/`

## 分层规则

- React 只处理界面状态、DTO 和 invoke 调用。
- React 不碰网络、持久化、凭据库、账号选择策略。
- Tauri command 只做薄桥接。
- 业务编排放 `src-core/src/application/`。
- 纯计算和选择策略放 `src-core/src/domain/`。
- HTTP、HTML 解析、文件、凭据、系统 API 放 `src-core/src/infrastructure/`。
- `domain` 里出现网络请求、文件读写、系统调用，直接判定分层打穿。

## 稳定链路

### 登录和切号

- 入口：`src-core/src/application/backend.rs`。
- 主要方法：`login_selected_account`、`login_selected_account_inner`。
- portal 登录由 `LegacyPortalAuthClient` 处理。
- 覆盖登录走 `srun_portal_pc.php?ac_id=1&` 表单。
- `/include/auth_action.php` 可返回 `IP has been online, please logout.`，不能当切号主链路。

### 刷新、流量和在线设备

- 入口：`run_refresh`、`refresh_inner`。
- 先查本机 IP。
- 再读 `srun_portal_pc_success.php` 成功页。
- 成功页解析本机 IP、上网用户、已用流量、计费方式。
- 自助面板 SSO URL 是 `traffic_portal_url` 的 origin 加 `/site/sso?data=base64(username:username)`。
- SSO 第一跳 302 设置的 `PHPSESSID_8800` cookie 必须保留。
- 再请求 `/home`，解析产品、流量、在线设备、下线链接。
- 不要回退到“带密码扫描所有账号”。慢，也容易打坏状态。

### 本机下线

- 入口：`logout_local_device_inner`。
- 只用于用户主动下号。
- 必须有本机 IP 和 `current_online_account_id`。
- 不用于切号前置步骤。

## 数据和凭据

- 密码不写回 JSON。
- 凭据继续走 `src-core/src/infrastructure/security/credential_vault.rs`。
- 改本地存储格式时，先补 `src-core/src/infrastructure/persistence/migration.rs`。
- 迁移要兼容旧用户数据。

## 修改流程

改动前：

- 读相关入口文件。
- 搜现有实现，别重复造轮子。
- 说明会碰到的目录、模块和入口。
- 跨 React、Tauri、Rust 三层以上时，先写清调用链。

改动时：

- 用最小改动。
- 不顺手重排无关代码。
- 不在界面里写解释性废话。
- 不留旧文件占位符。
- 不创建额外验证清单文档。

改动后：

- 按改动类型跑最小检查。
- 明确说哪些检查跑了，哪些没跑。

## 常用检查

- React：`pnpm build`、`pnpm lint`。
- Rust：`cargo fmt --check`、`cargo check`。
- 策略、解析、持久化、网络流程：再跑 `cargo test`。
- Windows 发布相关：必要时跑 `cargo build --release`。

## 回复规则

- 始终用中文。
- 结论先说。
- 只讲当前需要的信息。
- 不确定就标清依据和边界。
- 发现分层打穿、硬编码密码、手改生成物、把屎山继续堆大，直接指出。
