# 改桥接前检查

1. React 调用入口在哪。
2. `src/lib/muc.ts` 是否已有 invoke 封装。
3. Rust 用例是否已经在 `src-core/src/application/`。
4. Tauri command 是否只做转发和错误映射。
5. DTO 字段是否和 React 类型同步。
6. loading、error、snapshot 更新是否受影响。
7. 事件推送和同步返回是否一致。
8. Windows 托盘或窗口行为是否受影响。
