# Windows 相关模块

- `src-slint/`
当前 Slint 界面入口和 Windows 桌面运行目标。

- `credential_vault.rs`
处理 Windows 凭据库。

- `startup_service.rs`
处理启动项。当前仍在遗留 Tauri 依赖里，迁移时要拆适配。

- `runtime_paths.rs`
处理应用数据目录、资源目录、旧数据目录。

- `tauri.conf.json`
遗留桌面壳配置。不要把新功能继续绑到这里。
