# 发布前检查项

1. 前端检查通过：`npm run check`
2. Rust 编译通过：`cargo check`
3. 相关测试通过：`cargo test`
4. 如改前端构建链，再跑 `npm run build`
5. 如改桌面壳或打包相关，再跑 `npm run tauri -- build`
6. 检查自启、托盘、关闭到托盘是否受影响
7. 检查 Windows 凭据库和本地路径逻辑
