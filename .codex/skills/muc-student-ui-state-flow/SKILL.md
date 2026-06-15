---
name: muc-student-ui-state-flow
description: 规划或修改这个仓库的 Svelte 前端状态流、页面入口、组件职责、feature 落点和 Tauri 交互边界。用在新增前端功能、重构页面逻辑、拆 store、调整 `src/routes`/`src/lib/components`/`src/lib/features`/`src/lib/stores` 分工时。
---

# MUC Student UI State Flow

前端最容易烂成一锅。先判断职责，再动文件。

## 目录职责

- `src/routes/` 只放页面入口和布局
- `src/lib/components/` 放通用组件
- `src/lib/features/` 放功能逻辑
- `src/lib/stores/` 放共享状态和桥接动作
- `src/lib/types/` 放前端类型

## 当前核心状态入口

先读 `src/lib/stores/app.ts`。这里已经管了：

- `appSnapshot`
- `uiState`
- `dialogState`
- Tauri command 调用
- Tauri 事件监听
- 窗口关闭最小化逻辑

如果新需求还是应用级状态，优先延续这里的模式。只有当某个功能足够独立时，才拆到 `src/lib/features/<feature>`。

## 落点规则

1. 只是页面排版或入口切换。
改 `src/routes/`。

2. 只是通用展示组件。
改 `src/lib/components/`。

3. 这是某个功能的业务逻辑、表单处理、数据整理。
放 `src/lib/features/<feature>/`。

4. 这是多个组件都依赖的 UI 状态、快照、桥接动作。
放 `src/lib/stores/`。

## 硬规则

- 页面文件别直接堆业务。
- 不要把所有逻辑继续塞进 `components`。
- 前端类型统一落 `src/lib/types/app.ts` 或对应类型文件。
- 不要把说明性文字写进界面。

## 改动时要一起看

- `src/lib/stores/app.ts`
- `src/lib/types/app.ts`
- 相关页面组件
- 相关对话框组件

如果改的是数据字段，还要检查 Tauri DTO 是否同步。

## 参考

- 状态和组件职责：`references/frontend-boundaries.md`
- 当前 store 暴露能力：`references/app-store-map.md`
