# 目录边界

- `src/pages/`：React 页面。
- `src/components/`：React 组件。
- `src/hooks/`：React hook。
- `src/lib/`：前端工具、Tauri invoke 封装、DTO 类型。
- `src-tauri/src/lib.rs`：Tauri command、插件注册、核心装配。
- `src-tauri/src/platform.rs`：Windows 桌面适配。
- `src-tauri/src/plugins/`：Tauri 插件封装。
- `src-core/src/application/`：用例、服务、DTO、运行时编排。
- `src-core/src/domain/`：纯领域模型、纯策略、纯计算。
- `src-core/src/infrastructure/`：网络、解析、持久化、凭据、安全、系统适配。

# 禁区

- 不手改 `build/`、`dist/`、`target/`、`src-tauri/target/`。
- 不把网络、持久化、凭据库或账号选择策略塞进 React。
- 不把业务编排塞进 Tauri command。
- 不把 HTTP、文件、系统调用塞进 `domain`。
- 不把密码存回明文文件。
- 不在界面里写解释性废话。
