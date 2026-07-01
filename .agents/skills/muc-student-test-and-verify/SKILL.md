---
name: muc-student-test-and-verify
description: 为 MUC-student 改动选择最小验证命令，并解释已跑、未跑和失败原因。仅在完成代码修改、用户询问检查方案、CI/构建失败、或改动涉及 React、Rust 核心、Tauri、Windows 打包、持久化、网络流程时使用；纯阅读、命名规划、普通问答不触发。
---

# MUC Student Test And Verify

按改动类型选最小检查。不要用一堆命令掩盖没看懂问题。

## 验证矩阵

### 只改 React 页面、组件、hook、样式

- `pnpm build`
- `pnpm lint`

### 只改 Rust 业务核心

- `cargo fmt --check`
- `cargo check`

改了策略、解析、持久化、网络流程时，再跑：

- `cargo test`

需要看 Rust 覆盖率时，再跑：

- `cargo coverage-summary`

### 改 React 和 Rust/Tauri 对接

- `pnpm build`
- `pnpm lint`
- `cargo fmt --check`
- `cargo check`

改了 DTO、command、事件、持久化或迁移时，再跑：

- `cargo test`

### 改 Tauri、托盘、自启、Windows 行为、发布配置

- `cargo fmt --check`
- `cargo check`
- `cargo test`

发布前再考虑：

- `pnpm build`
- `cargo build --release`

### 改 Android 适配、Tauri mobile、Android 凭据或打包

- `pnpm tauri:android:check`

需要看 mobile 构建或真机流程时，再跑：

- `pnpm tauri:android:dev`
- `pnpm tauri:android:build`

## 输出规则

- 明确列出已跑命令和结果。
- 没跑的命令要说明原因。
- 失败时只先修相关失败，不顺手扩大范围。
- 不创建 md 验证清单文档。
- 覆盖率临时目录用完删除，尤其是 `target/llvm-cov-target` 和 `target/llvm-cov/`。

## 覆盖率策略

覆盖率不是常规检查。以下情况才看：

- 用户明确问覆盖率。
- 大量补测试后，需要确认新测试打到目标模块。
- parser、网络状态、持久化迁移、自动切号等高风险链路刚补回归测试。
- CI 或重构后怀疑测试没有覆盖关键路径。

默认只看摘要：

- `cargo coverage-summary`

需要定位遗漏分支时再看 HTML：

- `cargo coverage`

本机没有 `cargo-llvm-cov` 时，先安装：

- `cargo install cargo-llvm-cov --locked`

不要为了追数字写空断言。覆盖率只用于发现盲区，结论仍以行为测试和关键断言为准。

## 参考

- 按改动选命令：`references/verification-matrix.md`
- 覆盖率策略：`references/coverage.md`
