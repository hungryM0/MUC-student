# 目录边界

- `src/routes/` 只放页面入口和布局。
- `src/lib/components/` 放通用界面组件。
- `src/lib/features/` 放按功能拆开的前端逻辑。
- `src/lib/stores/` 放共享前端状态。
- `src/lib/types/` 放前端类型。
- `src-tauri/src/application/` 放用例、服务、DTO、运行时编排。
- `src-tauri/src/domain/` 放纯领域模型和策略。
- `src-tauri/src/infrastructure/` 放网络、OCR、解析、持久化、安全、系统适配。
- `src-tauri/src/adapters_tauri/` 放 Tauri 适配层。

# 禁区

- 不要手改 `.svelte-kit/`、`build/`、`src-tauri/gen/`。
- 不要把密码存回 `accounts.json` 或别的明文文件。
- 不要在 `domain` 放 HTTP、文件、系统调用。
- 不要在页面文件里堆业务流程。
- 不要把解释性废话写进界面。
