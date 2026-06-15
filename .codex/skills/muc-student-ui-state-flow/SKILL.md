---
name: muc-student-ui-state-flow
description: 规划或修改这个仓库的 React 前端状态流、应用壳、组件职责、feature 落点和 Tauri 交互边界。用在新增前端功能、重构页面逻辑、拆 store、调整 `src/App.tsx`/`src/lib/components`/`src/lib/features`/`src/lib/hooks`/`src/lib/stores` 分工时。
---

# MUC Student UI State Flow

前端最容易烂成一锅。先判断职责，再动文件。

## 目录职责

- `src/main.tsx` 只放 React 挂载入口和全局样式引入
- `src/App.tsx` 只放应用壳、页签切换、全局弹窗和状态分发
- `src/lib/components/` 放通用组件
- `src/lib/components/ui/` 放 shadcn/ui 基础组件。不要放业务
- `src/lib/features/` 放功能逻辑
- `src/lib/hooks/` 放 React hooks
- `src/lib/stores/` 放共享状态和桥接动作
- `src/lib/types/` 放前端类型

## 当前核心状态入口

先读 `src/lib/stores/app.ts`。这里已经管了：

- `appSnapshot`
- `uiState`
- `dialogState`
- `subscribeAppStore`
- `getAppStoreSnapshot`
- Tauri command 调用
- Tauri 事件监听
- 窗口关闭最小化逻辑

React 组件通过 `src/lib/hooks/use-app-store.ts` 订阅这个 store。

如果新需求还是应用级状态，优先延续这里的模式。只有当某个功能足够独立时，才拆到 `src/lib/features/<feature>`。

## 落点规则

1. 只是应用壳、页签切换或全局弹窗装配。
改 `src/App.tsx`。

2. 只是通用展示组件。
改 `src/lib/components/`。

3. 只是 shadcn/ui 基础组件。
改 `src/lib/components/ui/`。不要把业务塞进去。

4. 这是某个功能的业务逻辑、表单处理、数据整理。
放 `src/lib/features/<feature>/`。

5. 这是多个组件都依赖的 UI 状态、快照、桥接动作。
放 `src/lib/stores/`。

6. 这是 React 订阅或复用 hook。
放 `src/lib/hooks/`。

## 硬规则

- `src/App.tsx` 别直接堆业务。
- 不要把所有逻辑继续塞进 `components`。
- 不要把业务逻辑塞进 `components/ui`。
- 前端类型统一落 `src/lib/types/app.ts` 或对应类型文件。
- 不要把说明性文字写进界面。

## 改动时要一起看

- `src/lib/stores/app.ts`
- `src/lib/hooks/use-app-store.ts`
- `src/lib/types/app.ts`
- `src/App.tsx`
- 相关 feature 组件
- 相关对话框组件

如果改的是数据字段，还要检查 Tauri DTO 是否同步。

## 参考

- 状态和组件职责：`references/frontend-boundaries.md`
- 当前 store 暴露能力：`references/app-store-map.md`
