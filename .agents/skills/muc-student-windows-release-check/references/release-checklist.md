# 发布前检查项

1. Rust 格式：`cargo fmt --check`
2. Rust 编译检查：`cargo check`
3. Rust 测试：`cargo test`
4. 前端构建：`pnpm build`
5. 发布构建：`cargo build --release`
6. Windows 行为：托盘、自启、关闭到托盘、凭据读写、路径读写
