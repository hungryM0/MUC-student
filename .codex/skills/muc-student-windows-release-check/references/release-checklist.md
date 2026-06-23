# 发布前检查项

1. Slint 格式通过：`cargo fmt --check`
2. Slint 构建通过：`cargo check`
3. Slint 测试通过：`cargo test`
4. 发布构建通过：`cargo build --release`
5. 遗留 Rust 业务核心仍受影响时，在 `src-tauri/` 跑对应检查。
