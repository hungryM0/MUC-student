# 常见改动落点

## 新增 Slint 界面功能

- 界面结构、控件、布局、样式：`src-slint/ui/`
- global 状态和 callback 声明：`src-slint/ui/`
- 窗口创建、状态写入、callback 绑定：`src-slint/src/main.rs`
- 业务用例调用：先拆纯 Rust 服务，再从 `src-slint/src/main.rs` 调用

## 新增后端用例

- Slint 回调装配：`src-slint/src/main.rs`
- 遗留 Tauri 暴露：`src-tauri/src/adapters_tauri/`
- 用例编排：`src-tauri/src/application/`
- 纯规则：`src-tauri/src/domain/`
- 外部系统：`src-tauri/src/infrastructure/`

## 改存储结构

先改领域或 DTO 需要的数据结构。
再改 repository。
再补 `migration.rs`。
最后跑读取和写回相关检查。
