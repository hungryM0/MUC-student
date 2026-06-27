处理 MUC-student 的 React 页面、组件、hook、前端状态、Tauri invoke 封装、DTO 类型、窗口交互和视觉风格。

React 只做界面和桥接。不要让前端知道后端细节。

## 落点

- 页面：`src/pages/`
- 复用组件：`src/components/`
- hook：`src/hooks/`
- Tauri invoke、前端 DTO、窗口工具：`src/lib/`
- 全局样式：`src/index.css`

## 先读

- 页面入口：`src/main.tsx`
- 当前页面：`src/pages/home.tsx`、`src/pages/settings.tsx`、`src/pages/about.tsx`
- invoke 和 DTO：`src/lib/muc.ts`
- 窗口工具：`src/lib/window.ts`

## 规则

- React 不直接碰网络、文件、凭据库、账号选择策略。
- 后端字段通过 `src/lib/muc.ts` 收口。
- 视觉改动保持已有产品气质，不写解释性废话。
- 删除旧 UI 时，直接删干净，不留占位符垃圾。
- 改 DTO 时，同步查 Rust `src-core/src/application/dto.rs`。
