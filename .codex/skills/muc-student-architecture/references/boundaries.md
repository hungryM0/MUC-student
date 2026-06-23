# 目录边界

- `src-slint/ui/` 放 Slint 界面、组件和样式 token。
- `src-slint/src/main.rs` 放 Slint 窗口创建、global 写入、callback 绑定和后端调用装配。
- `src-slint/build.rs` 只编译 `.slint` 文件。
- `src-tauri/src/application/` 放用例、服务、DTO、运行时编排。
- `src-tauri/src/domain/` 放纯领域模型和策略。
- `src-tauri/src/infrastructure/` 放网络、解析、持久化、安全、系统适配。
- `src-tauri/src/adapters_tauri/` 放遗留 Tauri 适配层。

# 禁区

- 不要手改 `build/`、`src-tauri/gen/`。
- 不要手改 `src-slint/target/`。
- 不要恢复 React/Vite/npm 旧栈。
- 不要把密码存回 `accounts.json` 或别的明文文件。
- 不要在 `domain` 放 HTTP、文件、系统调用。
- 不要在 `.slint` 里写网络、持久化和账号选择策略。
- 不要把解释性废话写进界面。
