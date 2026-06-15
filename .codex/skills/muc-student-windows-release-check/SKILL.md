---
name: muc-student-windows-release-check
description: 检查这个仓库在 Windows 下的构建、运行、自启、托盘、路径和 CI 兼容性。用在改系统行为、启动项、托盘、凭据库、Tauri 打包、GitHub Actions，或发布前想确认不会只在本机能跑的时候。
---

# MUC Student Windows Release Check

这个项目的 CI 跑 Windows。改了平台行为，就按 Windows 来想。

## 先看哪些模块

- `src-tauri/src/infrastructure/system/`
- `src-tauri/src/infrastructure/security/credential_vault.rs`
- `src-tauri/src/infrastructure/persistence/runtime_paths.rs`
- `src-tauri/tauri.conf.json`
- `.github/workflows/ci.yml`

## 重点检查项

### 构建

- `npm run build`
- `npm run tauri -- build`
- `cargo check`
- `cargo test`

### Windows 特有行为

- Windows 凭据库是否还能正常读写
- 自启开关是否还走 `StartupService`
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
