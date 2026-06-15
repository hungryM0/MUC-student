# 常见改动落点

## 新增前端功能

- 页面入口变更：`src/routes/`
- 状态和调用：`src/lib/stores/`
- 业务逻辑：`src/lib/features/<feature>/`
- 通用 UI：`src/lib/components/`
- 类型：`src/lib/types/`

## 新增后端用例

- Tauri 暴露：`src-tauri/src/adapters_tauri/`
- 用例编排：`src-tauri/src/application/`
- 纯规则：`src-tauri/src/domain/`
- 外部系统：`src-tauri/src/infrastructure/`

## 改存储结构

先改领域或 DTO 需要的数据结构。
再改 repository。
再补 `migration.rs`。
最后跑读取和写回相关检查。
