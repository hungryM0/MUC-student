# MUC-student 协作说明

这个仓库是给 MUC 校园网多账号拼车用的 Windows 桌面应用。界面壳已切到 Tauri v2 + React。后端核心在 Rust。

无登录态的时候，直接走 HTTP 登录就可以了。但是已经在登录态的时候，想要切号、查流量、下号都要走 OCR 登录页。

## 目录边界

- `src-core/` 是可复用业务核心。
- `src-core/src/application/` 放用例、服务、DTO、平台抽象和运行时编排。
- `src-core/src/domain/` 放领域模型和策略。这里别塞网络请求和文件读写。
- `src-core/src/infrastructure/` 放网络、OCR、解析、持久化、安全、系统适配。
- `src-tauri/` 是 Tauri 桌面壳。
- `src-tauri/src/lib.rs` 放 Tauri command、插件注册和核心装配。
- `src-tauri/src/platform.rs` 放桌面平台适配，比如运行路径和 HKCU Run 自启。
- `src-tauri/src/plugins/` 放 Tauri 插件封装。
- `src/` 是 React 前端。
- `src/pages/` 放页面。
- `src/components/` 放组件。
- `src/lib/` 放前端工具、Tauri invoke 封装和 DTO 类型。
- `target/`、`dist/`、`src-tauri/target/` 是生成物。除非任务明确要求，否则别手改。

## 已验证命令

根目录：

- `pnpm build`
- `cargo fmt --check`
- `cargo check`
- `cargo test`

CI 在 `.github/workflows/ci.yml`，跑的是 Windows。改动如果会影响平台行为，先按 Windows 约束想。

## 代码规则

- 先读相关文件，再改。不要看个文件名就瞎写。
- 小改跟着现有职责走。大改先定边界，再拆模块。
- React 前端只读写 DTO 和 invoke 封装，不直接碰网络、持久化、凭据库或账号选择策略。
- Tauri command 只做薄桥接，业务编排继续放 `src-core/src/application/`。
- WinRT/Win32 API 调用返回 `Result` 时必须处理错误，不要在 UI/系统调用链里 `unwrap()`。
- Rust 分层别打穿。`domain` 保持纯，`application` 编排流程，`infrastructure` 处理外部系统。
- 账号、状态、迁移、凭据存储这几块已经分开了。别把密码写回 JSON。凭据继续走 `credential_vault`。
- 旧数据迁移在 `src-core/src/infrastructure/persistence/migration.rs`。改本地存储格式时，先补迁移，再谈别的。
- OCR provider 有固定链路：`NativeRustOcrProvider` 然后 `ExternalWorkerOcrProvider`。别偷改顺序。
- 不要把说明性废话写进界面。

## 工作方法

- 改动前先列出会碰到的目录、模块和入口。
- 涉及界面和后端联动时，先确认接口落点：React 事件、Tauri command、用例、基础设施各在哪。
- 涉及持久化、自动切号、在线设备、托盘、自启时，先搜现有实现，别重复造轮子。
- 编辑前先读。写文件时用最小改动，不顺手重排无关代码。
- 改完至少跑相关检查。界面或业务改动先在根目录跑 `pnpm build` 和 `cargo check`，能测再跑 `cargo test`。

## 沟通规则

- 用中文。
- 结论先说，废话删掉。
- 不确定就标清依据和边界。
- 发现目录职责混乱、分层打穿、硬编码密码、把生成物当源码改，直接指出来。
