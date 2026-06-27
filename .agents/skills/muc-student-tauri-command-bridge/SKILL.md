---
name: muc-student-tauri-command-bridge
description: 处理 MUC-student 的 Tauri command、事件、DTO、托盘和窗口行为桥接。仅在改 `src-tauri/src/lib.rs`、`tauri.conf.json`、Tauri command、事件 emit、DTO 兼容、托盘/窗口行为时使用；纯 React UI、纯 Rust service、纯 parser 改动不触发。
---

# MUC Student Tauri Command Bridge

当前桌面壳是 Tauri v2 + React。Tauri command 只做薄桥接，业务编排放 `src-core/src/application/`。

## 当前桥接

- Tauri command 在 `src-tauri/src/lib.rs`
- 用例在 `src-core/src/application/backend.rs`
- DTO 在 `src-core/src/application/dto.rs`
- 前端 invoke 封装在 `src/lib/muc.ts`
- 事件名包括 `muc://state-updated`、`muc://task-started`、`muc://task-finished`

## 先读

- `src/lib/muc.ts`
- `src-tauri/src/lib.rs`
- `src-core/src/application/backend.rs`
- `src-core/src/application/dto.rs`

## 改 command 的顺序

1. 先找 React 调用点。
2. 再找 `src/lib/muc.ts` 的 invoke 封装。
3. 找 `AppCore` 里的实际用例。
4. 找 DTO 输入输出。
5. Tauri command 只做参数转发和错误映射。

## DTO 规则

- React 字段看 `src/lib/muc.ts` 和页面组件。
- Rust 输入输出看 `src-core/src/application/dto.rs`。
- 改字段时，React 类型和 Rust DTO 一起查。

## 事件规则

Tauri 事件：

- `muc://state-updated`
- `muc://task-started`
- `muc://task-finished`

## 常见坑

- 业务逻辑塞进 Tauri command。
- Rust DTO 变了，React 类型没变。
- 事件和同步返回状态不一致。
- 把业务直接塞进桥接层，桥接层变成屎山。

## 参考

- 桥接检查清单：`references/bridge-checklist.md`
- 当前 command 和事件地图：`references/current-bridge-map.md`
