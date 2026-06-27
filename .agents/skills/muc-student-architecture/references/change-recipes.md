# 常见改动落点

## 新增 React 界面功能

- 页面结构、控件、布局、样式：`src/pages/` 或 `src/components/`
- 前端 DTO 和 invoke 封装：`src/lib/`
- Tauri command 暴露：`src-tauri/src/lib.rs`
- 业务用例调用：先拆 `src-core/src/application/` 服务，再从 Tauri command 调用

## 新增后端用例

- React invoke 封装：`src/lib/`
- Tauri command 暴露：`src-tauri/src/lib.rs`
- 用例编排：`src-core/src/application/`
- 纯规则：`src-core/src/domain/`
- 外部系统：`src-core/src/infrastructure/`

## 改存储结构

先改领域或 DTO 需要的数据结构。
再改 repository。
再补 `migration.rs`。
最后跑读取和写回相关检查。
