---
name: muc-student-windows-release-check
description: 检查 MUC-student 在 Windows 下的 Tauri 构建、运行、自启、托盘、路径、凭据库和 CI 兼容性。仅在改 `src-tauri` Windows 行为、`credential_vault.rs`、`runtime_paths.rs`、`tauri.conf.json`、GitHub Actions、打包发布流程，或用户明确要求发布前 Windows 检查时使用。
---

# MUC Student Windows Release Check

这个项目的 CI 跑 Windows。改了平台行为，就按 Windows 来想。

## 先看哪些模块

- `src-tauri/src/platform.rs`
- `src-core/src/infrastructure/security/credential_vault.rs`
- `src-core/src/infrastructure/persistence/runtime_paths.rs`
- `src-tauri/tauri.conf.json`
- `.github/workflows/ci.yml`

## 重点检查项

### 构建

- `cargo fmt --check`
- `cargo check`
- `cargo test`
- `cargo build --release`

### Windows 特有行为

- Windows 凭据库是否还能正常读写
- 自启开关是否还走 Windows 注册表适配
- 托盘和关闭到托盘逻辑是否还通
- 路径拼接是否依赖 Windows 目录语义

### CI

- 改动会不会影响 Windows runner
- 新命令或新依赖是否能在 CI 上装起来

## 硬规则

- 涉及凭据存储时，继续走 Windows 凭据库。
- 不要把只在本机成立的相对路径写死。
- 不要改出 Linux/macOS 才能跑的命令。

## 参考

- 发布前检查项：`references/release-checklist.md`
- Windows 相关模块：`references/windows-modules.md`
