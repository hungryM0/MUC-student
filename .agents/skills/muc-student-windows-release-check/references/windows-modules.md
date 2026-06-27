# Windows 相关模块

- `src-tauri/src/platform.rs`
  处理运行路径和 HKCU Run 自启。

- `src-tauri/src/plugins/system_tray.rs`
  处理托盘菜单和窗口显示隐藏。

- `src-core/src/infrastructure/security/credential_vault.rs`
  处理 Windows 凭据库。

- `src-core/src/infrastructure/persistence/runtime_paths.rs`
  处理应用数据目录、资源目录、旧数据目录。

- `src-tauri/tauri.conf.json`
  处理桌面壳、打包、窗口、插件配置。

- `.github/workflows/ci.yml`
  处理 Windows CI。
