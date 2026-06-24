---
name: muc-student-ui-state-flow
description: 处理 MUC-student 的 React 前端状态、页面和 Tauri invoke 封装。仅在改 `src/pages/`、`src/components/`、`src/lib/`、React UI 状态、窗口交互或视觉风格时使用；纯 Rust 业务、纯 parser、纯持久化改动不触发。
---

# MUC Student React UI Flow

当前界面入口是 Tauri v2 + React。

## 规则

- 页面放 `src/pages/`。
- 组件放 `src/components/`。
- Tauri invoke 封装和前端 DTO 放 `src/lib/`。
- React 不直接碰网络、持久化、凭据库或账号选择策略。
- 不要把说明性废话写进界面。
