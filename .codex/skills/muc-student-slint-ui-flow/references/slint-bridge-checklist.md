# Slint 对接清单

1. 用户动作是否已有 callback。
2. callback 名是否只表达动作。
3. Rust 绑定是否在 `src-slint/src/main.rs`。
4. 后端调用是否绕开 `.slint`。
5. 后端返回值是否映射成界面需要的 global。
6. loading、错误、空状态是否有明确字段。
7. 是否误用了 Tauri `emit`、`invoke`、`AppHandle`。
8. 是否需要先拆 `src-tauri/src/application/backend.rs` 的 Tauri 依赖。
