规划或审查 MUC-student 的分层边界、模块落点和跨层改动。

先读仓库根目录 `CLAUDE.md`，再读本次会碰到的入口文件。

## 落点判断

- React 页面、布局：`src/pages/`
- React 复用组件：`src/components/`
- React hook：`src/hooks/`
- 前端 DTO、Tauri invoke、窗口工具：`src/lib/`
- Tauri command、插件注册、核心装配：`src-tauri/src/lib.rs`
- Windows 桌面适配：`src-tauri/src/platform.rs`
- Tauri 插件封装：`src-tauri/src/plugins/`
- 用例、服务、DTO、运行时编排：`src-core/src/application/`
- 领域模型、纯策略、纯计算：`src-core/src/domain/`
- 网络、解析、持久化、凭据、安全、系统适配：`src-core/src/infrastructure/`

## 稳定期规则

- 默认小改。不为"更优雅"重写稳定链路。
- 跨三层以上时，先列调用链，再改代码。
- 改存储格式时，先补迁移。
- 改 DTO 时，同时查 React 类型和 Rust DTO。
- 改策略时，先确认策略是否属于 `domain`，外部 IO 不准跟进去。

## 硬禁区

- 不手改 `build/`、`dist/`、`target/`、`src-tauri/target/`。
- 不把业务网络、持久化、凭据库或账号选择策略塞进 React。
- 不把业务编排塞进 Tauri command。
- 不把 HTTP、文件读写、系统 API 塞进 `domain`。
- 不把密码写回 JSON。
- 不在界面里写解释性废话。

## 调用链查法

`src/pages/*` → `src/lib/muc.ts` → `src-tauri/src/lib.rs` → `src-core/src/application/` → `src-core/src/domain/` 或 `src-core/src/infrastructure/`

先找已有入口。找不到再新建。
