# 改遗留桥接前检查

1. 这次是否真的需要保留 Tauri。
2. 如果迁到 Slint，Slint callback 在哪。
3. Rust 用例入口在哪。
4. 用例是否依赖 `tauri::AppHandle`、`emit`、Tauri path API。
5. DTO 字段是否和 Slint global 同步。
6. loading、error、snapshot 更新是否受影响。
