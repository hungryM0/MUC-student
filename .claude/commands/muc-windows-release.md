检查 MUC-student 在 Windows 下的 Tauri 构建、运行、自启、托盘、路径、凭据库、更新器、CI 和发布包兼容性。

目标平台是 Windows。平台相关改动按 Windows 语义判断。

## 先读

- Tauri 壳：`src-tauri/src/lib.rs`
- Windows 平台适配：`src-tauri/src/platform.rs`
- 托盘：`src-tauri/src/plugins/system_tray.rs`
- 凭据库：`src-core/src/infrastructure/security/credential_vault.rs`
- 运行路径：`src-core/src/infrastructure/persistence/runtime_paths.rs`
- Tauri 配置：`src-tauri/tauri.conf.json`
- CI：`.github/workflows/ci.yml`

## 检查重点

- Windows 凭据库读写是否还走 `credential_vault`。
- 自启是否还走 HKCU Run。
- 托盘、关闭到托盘、窗口恢复是否还通。
- 路径拼接是否依赖正确的 Windows 目录。
- 新依赖和命令是否能在 Windows runner 上跑。
- 发布包是否仍能包含前端构建产物。

## 验证

常规平台改动：
```
cargo fmt --check
cargo check
cargo test
```
发布前按需加：
```
pnpm build
cargo build --release
```

## 硬规则

- 不把凭据改回明文文件。
- 不写死只在本机成立的绝对路径。
- 不引入 Linux/macOS 专属命令到 Windows CI。
