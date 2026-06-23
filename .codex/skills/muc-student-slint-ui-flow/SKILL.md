---
name: muc-student-slint-ui-flow
description: 规划或修改 MUC-student 的 Slint/WinUI 3 界面、`.slint` 组件、Slint global 状态、Rust 回调绑定和界面到后端调用边界。仅在改 `src-slint/ui/*.slint`、`src-slint/src/main.rs`、Slint UI 状态、窗口交互、界面回调或视觉风格时使用；纯网络、纯 parser、纯持久化改动不触发。
---

# MUC Student Slint UI Flow

当前界面入口是 `src-slint/`。目标是贴近 WinUI 3 的 Windows 桌面应用，不要恢复 React、Vite、Tauri WebView、shadcn 或 Fluent React。

## 先读什么

- `src-slint/ui/app.slint`
- `src-slint/src/main.rs`
- `src-slint/build.rs`
- 需要对接业务时，再读 `src-tauri/src/application/dto.rs` 和相关用例。

Slint 官方 Rust 模式是：`build.rs` 用 `slint_build::compile("ui/app.slint")` 编译界面，Rust 入口用 `slint::include_modules!()` 引入生成模块。

## 目录职责

- `src-slint/ui/`：Slint markup。放组件、布局、样式 token、global 状态和 callback 声明。
- `src-slint/src/main.rs`：创建窗口、写入 global 状态、绑定 callback、调用 Rust 后端。
- `src-slint/build.rs`：只编译 `.slint` 文件。
- `src-tauri/src/application/`：暂存旧应用服务和 DTO。迁移时先去 Tauri 依赖，再给 Slint 调。
- `src-tauri/src/domain/`：纯模型和策略。
- `src-tauri/src/infrastructure/`：网络、解析、持久化、凭据、系统适配。

## 落点规则

1. 只是窗口布局、导航、卡片、按钮、输入框、列表。
改 `src-slint/ui/app.slint`，或在 `src-slint/ui/` 拆新 `.slint` 文件。

2. 是界面共享状态。
优先放 Slint `global`。Rust 侧通过 `ui.global::<...>()` 读写。

3. 是用户点击、提交、刷新、删除等意图。
在 `.slint` 声明 callback，在 `src-slint/src/main.rs` 绑定处理。

4. 是网络登录、流量查询、自动切号、持久化。
不要写进 `.slint`。接到 Rust 用例。

5. 是样式系统。
用少量共享色值、间距、圆角和字号，贴近 WinUI 3：克制、清晰、密度适中。不要堆解释性文案。

## 硬规则

- 不要恢复 `src/`、`package.json`、Vite、React、shadcn、Fluent React。
- 不要把网络请求、文件读写、账号选择策略写进 `.slint`。
- 不要把说明性废话写进界面。
- 不要手改 `src-slint/target/`。
- 新增 `.slint` 文件后确认 `build.rs` 会编译入口文件。
- Slint 回调名用动词短语，表示用户动作，不表示实现细节。

## 对接后端

现状里 `src-tauri/src/application/backend.rs` 还强依赖 `tauri::AppHandle`、事件 emit 和 Tauri 路径 API。这块是脏点。迁移到 Slint 时先拆出不依赖 Tauri 的应用服务，再让 Slint 调它。

迁移顺序：

1. 找当前 Slint callback。
2. 找需要调用的业务用例。
3. 如果用例依赖 Tauri，先拆路径、事件、自启适配。
4. 让 `src-slint/src/main.rs` 调纯 Rust 服务。
5. 把结果映射回 Slint global。

## 验证

- 只改 Slint：在 `src-slint/` 跑 `cargo fmt --check`、`cargo check`。
- 改业务核心：在对应 Rust crate 跑 `cargo fmt --check`、`cargo check`，能测再跑 `cargo test`。
- 改 Windows 行为或发布流程：同时看 `muc-student-windows-release-check`。

## 参考

- Slint 目录边界：`references/slint-boundaries.md`
- Slint 对接清单：`references/slint-bridge-checklist.md`
