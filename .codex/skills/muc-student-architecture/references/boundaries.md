# 目录边界

- `src/pages/` 放 React 页面。
- `src/components/` 放 React 组件。
- `src/lib/` 放前端工具、Tauri invoke 封装和 DTO 类型。
- `src-tauri/src/lib.rs` 放 Tauri command、插件注册和核心装配。
- `src-tauri/src/platform.rs` 放桌面平台适配。
- `src-tauri/src/plugins/` 放 Tauri 插件封装。
- `src-core/src/application/` 放用例、服务、DTO、运行时编排。
- `src-core/src/domain/` 放纯领域模型和策略。
- `src-core/src/infrastructure/` 放网络、解析、持久化、安全、系统适配。

# 禁区

- 不要手改 `build/`、`dist/`、`target/`、`src-tauri/target/`。
- 不要把网络、持久化、凭据库或账号选择策略塞进 React。
- 不要把密码存回 `accounts.json` 或别的明文文件。
- 不要在 `domain` 放 HTTP、文件、系统调用。
- 不要把解释性废话写进界面。
