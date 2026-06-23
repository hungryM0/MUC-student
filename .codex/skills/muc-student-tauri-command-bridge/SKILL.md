---
name: muc-student-tauri-command-bridge
description: 处理 MUC-student 遗留 Tauri command、事件、DTO、托盘和窗口行为桥接，或把旧 Tauri bridge 迁到 Slint/Rust 调用。仅在改 `src-tauri/src/adapters_tauri`、`tauri.conf.json`、Tauri command、事件 emit、DTO 兼容、托盘/窗口行为，或拆除旧 Tauri 依赖时使用；纯 Slint UI、纯 Rust service、纯 parser 改动不触发。
---

# MUC Student Tauri Command Bridge

这是遗留 WebView 壳。当前界面已转向 `src-slint/`，不要新增旧 `invoke` 链路。

## 当前遗留桥

- Tauri command 在 `src-tauri/src/lib.rs`
- 用例在 `src-tauri/src/application/backend.rs`
- DTO 在 `src-tauri/src/application/dto.rs`
- 事件名包括 `app://state-updated`、`app://task-started`、`app://task-finished`

## 当前 Rust 入口

先读：

- `src-tauri/src/adapters_tauri/`
- `src-tauri/src/application/backend.rs`
- `src-tauri/src/application/dto.rs`

## 改 command 的顺序

1. 先判断是不是还应该保留 Tauri。
2. 如果是迁到 Slint，先找 `src-slint/src/main.rs` 的 callback。
3. 找 `Backend` 里的实际用例。
4. 找 DTO 输入输出。
5. 拆掉 `tauri::AppHandle`、`emit`、Tauri path API 后再给 Slint 调。

## DTO 规则

- Slint 字段看 `src-slint/ui/*.slint`
- Rust 输入输出看 `application/dto.rs` 和 adapter 的反序列化结构
- 改字段时，Slint global 和 Rust DTO 一起查

## 事件规则

遗留 Tauri 事件：

- `app://state-updated`
- `app://log-appended`
- `app://task-started`
- `app://task-finished`

迁到 Slint 时，不要把这些事件照搬成必需设计。优先直接写 global 状态。

## 常见坑

- 继续给新 Slint 功能加 Tauri command。
- Rust DTO 变了，Slint global 没变。
- 后端还在 emit 事件，Slint 侧却等同步返回。
- 把业务直接塞进 adapter，桥接层变成屎山。

## 参考

- 桥接检查清单：`references/bridge-checklist.md`
- 当前 command 和事件地图：`references/current-bridge-map.md`
