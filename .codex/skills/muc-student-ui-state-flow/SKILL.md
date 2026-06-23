---
name: muc-student-ui-state-flow
description: 已废弃。仅在阅读或删除 MUC-student 旧 React 前端遗留文件时使用。新界面工作必须使用 `muc-student-slint-ui-flow`，不要恢复 `src/App.tsx`、`src/lib/features`、`src/lib/stores`、React、Vite、shadcn 或 Fluent React。
---

# MUC Student UI State Flow

这个 skill 只记录旧 React 前端的废弃状态。新工作转到 `muc-student-slint-ui-flow`。

## 硬规则

- 不要新增 React 文件。
- 不要恢复 npm/Vite 构建链。
- 如果发现旧 React 文件还在，先确认是否仍被引用。未引用就删掉，不留占位垃圾。
- 如果要做新 UI，切到 `muc-student-slint-ui-flow`。
