---
name: muc-student-tauri-command-bridge
description: 处理 MUC-student 的 Tauri v2 command、事件、DTO、托盘、窗口行为和 React invoke 桥接。仅在改 `src-tauri/src/lib.rs`、`src-tauri/src/platform.rs`、`src-tauri/src/plugins/`、`tauri.conf.json`、`src/lib/muc.ts`、command、事件 emit、DTO 兼容、托盘或窗口行为时使用；纯 React 样式、纯 Rust service、纯 parser 改动不触发。
---

# MUC Student Tauri Command Bridge

Tauri command 是薄桥接。业务编排继续放 `src-core/src/application/`。

## 先读

- React 调用：`src/lib/muc.ts`。
- Tauri command：`src-tauri/src/lib.rs`。
- 后端用例：`src-core/src/application/backend.rs`。
- Rust DTO：`src-core/src/application/dto.rs`。
- 托盘：`src-tauri/src/plugins/system_tray.rs`。
- Windows 平台适配：`src-tauri/src/platform.rs`。

## 改 command 的顺序

1. 找 React 调用点。
2. 查 `src/lib/muc.ts` 是否已有 invoke 封装。
3. 查 `AppCore` 或 `Backend` 是否已有用例。
4. 查输入输出 DTO。
5. 只在 Tauri command 做参数转发和错误映射。

## DTO 和事件规则

- React 类型和 Rust DTO 要同步。
- 字段改名时，查页面组件、`src/lib/muc.ts`、`src-core/src/application/dto.rs`。
- 同步返回状态和事件推送不能互相矛盾。

当前事件：

- `muc://state-updated`
- `muc://task-started`
- `muc://task-finished`

## 常见坑

- 把业务逻辑塞进 `src-tauri/src/lib.rs`。
- Rust DTO 改了，React 类型没改。
- command 返回状态和事件状态不一致。
- 托盘或窗口行为只按本机手感改，没考虑 Windows CI 和发布包。

## 参考

- 桥接检查清单：`references/bridge-checklist.md`
- 当前 command 和事件地图：`references/current-bridge-map.md`
