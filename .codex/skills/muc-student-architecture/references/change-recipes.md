# 常见改动落点

## 新增前端功能

- 应用壳、页签、全局弹窗：`src/App.tsx`
- 状态和调用：`src/lib/stores/`
- React hooks：`src/lib/hooks/`
- 业务逻辑：`src/lib/features/<feature>/`
- 通用 UI：`src/lib/components/`
- shadcn/ui 基础组件：`src/lib/components/ui/`
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
