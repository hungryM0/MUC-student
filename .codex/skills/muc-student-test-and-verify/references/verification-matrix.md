# 验证矩阵

## Slint 界面、global、callback

- 在 `src-slint/` 跑 `cargo fmt --check`
- 在 `src-slint/` 跑 `cargo check`
- 影响回调绑定或构建链时加 `cargo test`

## Rust 领域、应用、基础设施

- `cargo check`
- 有相关测试就跑 `cargo test`

## Slint 和 Rust 对接

- `cargo check`
- 必要时 `cargo test`

## 遗留 Tauri 打包、托盘、自启、Windows 行为

- `cargo check`
- `cargo test`
