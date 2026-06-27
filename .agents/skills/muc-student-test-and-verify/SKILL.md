---
name: muc-student-test-and-verify
description: 为 MUC-student 改动选择最小验证命令，并解释已跑/未跑的检查。仅在完成代码修改、用户询问该跑什么检查、CI/构建失败、或改动涉及 React 界面、Rust 业务核心、Tauri、Windows 打包时使用；纯代码阅读、命名规划、普通问答不触发。
---

# MUC Student Test And Verify

改完别拍脑袋说“应该没事”。按改动类型选检查。

## 最小验证集

### 只改 React 界面

- `pnpm build`
- `pnpm lint`

### 只改 Rust 业务

- `cargo check`

如果改了策略、解析、持久化、网络流程，能测就再跑：

- `cargo test`

### 改了 React 和 Rust 对接

- `pnpm build`
- `pnpm lint`
- `cargo fmt --check`
- `cargo check`

改了 DTO、command、持久化或迁移链路，再加：

- `cargo test`

### 改了 Tauri 或 Windows 行为

- `cargo fmt --check`
- `cargo check`
- `cargo test`

## 使用规则

- 先按最小集跑。
- 如果失败，先修相关失败，不要顺手扩大范围瞎跑。
- 结果里明确说哪些跑了，哪些没跑。

## 参考

- 按改动选命令：`references/verification-matrix.md`
