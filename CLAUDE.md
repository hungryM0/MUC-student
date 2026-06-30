# MUC-student 协作说明

MUC 校园网多账号工具，Tauri v2 + React + Rust，目标平台 Windows 优先，Android 已接入。

项目已进入稳定期。默认小步、低风险改动。先读现有实现，再判断落点。不要重写稳定链路。

## 架构

- 桌面壳：Tauri v2，`src-tauri/`。
- 前端：React，`src/`。
- 后端核心：Rust，`src-core/`。
- CI：`.github/workflows/ci.yml`，Windows 约束。Android 相关命令走本地环境检查。

目录边界：

- `src/pages/`：页面。
- `src/components/`：组件。
- `src/hooks/`：前端 hook。
- `src/lib/`：Tauri invoke 封装、前端 DTO、窗口工具。
- `src-tauri/src/lib.rs`：command、插件注册、核心装配。
- `src-tauri/src/platform.rs`：Windows 运行路径和 HKCU Run 自启。
- `src-tauri/src/plugins/`：Tauri 插件封装。
- `src-tauri/gen/android/`：Android Gradle 工程和 Tauri 生成物。
- `src-core/src/application/`：用例、服务、DTO、运行时编排。
- `src-core/src/domain/`：领域模型和纯策略。
- `src-core/src/infrastructure/`：网络、解析、持久化、凭据、安全、系统适配。
- Android 端也守这条边界。平台差异放适配层，不要往 React 或 `domain` 里塞条件分支。

不手改生成物：`build/`、`dist/`、`target/`、`src-tauri/target/`。

## 分层规则

- React 只处理界面状态、DTO 和 invoke 调用。
- React 不碰网络、持久化、凭据库、账号选择策略。
- Tauri command 只做薄桥接。
- 业务编排放 `src-core/src/application/`。
- 纯计算和选择策略放 `src-core/src/domain/`。
- HTTP、HTML 解析、文件、凭据、系统 API 放 `src-core/src/infrastructure/`。
- `domain` 里出现网络请求、文件读写、系统调用，直接判定分层打穿。

## 登录态规则

- 无登录态时，直接走 portal HTTP 登录。
- 已登录时，切号走登录页表单覆盖登录。
- 查流量、在线设备、本机下线，优先走成功页和 SSO 自助面板链路。
- 不要把切号改成"先下线再登录"。这是错路。
- `/include/auth_action.php` 可返回 `IP has been online, please logout.`，不能当切号主链路。
- Android 侧如果有平台差异，优先放适配层，不改稳定链路语义。

## 稳定链路入口

- 登录和切号：`Backend::login_selected_account`、`login_selected_account_inner`。
- 刷新状态：`Backend::run_refresh`、`refresh_inner`。
- 本机下线：`Backend::logout_local_device_inner`。
- 调用链：`src/pages/*` → `src/lib/muc.ts` → `src-tauri/src/lib.rs` → `src-core/src/application/` → `domain` 或 `infrastructure`。

## 数据和凭据

- 密码不写回 JSON。
- 凭据走 `src-core/src/infrastructure/security/credential_vault.rs`。
- 改本地存储格式时，迁移逻辑写在 `src-core/src/infrastructure/persistence/database.rs` 的 `user_version` 分支里。
- Android 凭据继续走平台 keyring，不要改成明文文件。

## 修改流程

改动前：读相关入口文件，搜现有实现，说明涉及目录/模块/入口，跨三层以上先列调用链。

改动时：最小改动，不顺手重排无关代码，不在界面写解释性文字，不留旧文件占位符。

改动后：按改动类型跑最小检查，明确说哪些跑了、哪些没跑。

## 常用检查

- 只改 React：`pnpm build`、`pnpm lint`。
- 只改 Rust：`cargo fmt --check`、`cargo check`。
- 改了策略/解析/持久化/网络流程：再跑 `cargo test`。
- 发布相关：必要时 `cargo build --release`。
- Android 适配相关：优先跑 `pnpm tauri:android:check`，需要时再跑 `pnpm tauri:android:dev` 或 `pnpm tauri:android:build`。

## 回复规则

- 始终用中文。
- 结论先说。
- 只讲当前需要的信息。
- 不确定就标清依据和边界。
- 发现分层打穿、硬编码密码、手改生成物，直接指出。
- 工作完成后不创建 md 验证清单文档。
