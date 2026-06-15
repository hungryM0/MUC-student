---
name: muc-student-test-and-verify
description: 根据这个仓库的改动类型选择最小必要检查命令，并在改完后补跑前端、Rust、Tauri 和 CI 相关验证。用在不确定该跑 `npm run check`、`cargo check`、`cargo test`、`npm run build` 还是 `npm run tauri -- build`，或需要收敛验证范围的时候。
---

# MUC Student Test And Verify

改完别拍脑袋说“应该没事”。按改动类型选检查。

## 最小验证集

### 只改前端

- `npm run check`

如果改了构建链、Vite 配置、React 入口、Tailwind/shadcn 配置或静态资源，再加：

- `npm run build`

### 只改 Rust

- `cargo check`

如果改了策略、解析、持久化、网络流程，能测就再跑：

- `cargo test`

### 改了前后端桥接

- `npm run check`
- `cargo check`

如果改了 DTO、command、事件或 Tauri 配置，再考虑：

- `cargo test`
- `npm run tauri -- build`

### 改了桌面壳或 Windows 行为

- `npm run check`
- `cargo check`
- `cargo test`
- `npm run tauri -- build`

## 使用规则

- 先按最小集跑。
- 如果失败，先修相关失败，不要顺手扩大范围瞎跑。
- 结果里明确说哪些跑了，哪些没跑。

## 参考

- 按改动选命令：`references/verification-matrix.md`
