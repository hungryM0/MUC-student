---
name: muc-student-test-and-verify
description: 为 MUC-student 改动选择最小验证命令，并解释已跑/未跑的检查。仅在完成代码修改、用户询问该跑什么检查、CI/构建失败、或改动涉及 Slint 界面、Rust 业务核心、遗留 Tauri、Windows 打包时使用；纯代码阅读、命名规划、普通问答不触发。
---

# MUC Student Test And Verify

改完别拍脑袋说“应该没事”。按改动类型选检查。

## 最小验证集

### 只改 Slint 界面

在 `src-slint/` 目录跑：

- `cargo fmt --check`
- `cargo check`

如果改了回调绑定或构建链，再加：

- `cargo test`

### 只改 Rust 业务

- `cargo check`

如果改了策略、解析、持久化、网络流程，能测就再跑：

- `cargo test`

### 改了 Slint 和 Rust 对接

- `cargo fmt --check`
- `cargo check`

分别在受影响 crate 跑。改了 DTO、回调或迁移链路，再考虑：

- `cargo test`

### 改了遗留 Tauri 或 Windows 行为

- `cargo check`
- `cargo test`

## 使用规则

- 先按最小集跑。
- 如果失败，先修相关失败，不要顺手扩大范围瞎跑。
- 结果里明确说哪些跑了，哪些没跑。

## 参考

- 按改动选命令：`references/verification-matrix.md`
