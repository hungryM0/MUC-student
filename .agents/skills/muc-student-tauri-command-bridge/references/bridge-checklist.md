# 改桥接前检查

1. React 调用入口在哪。
2. `src/lib/muc.ts` 是否已有 invoke 封装。
3. Rust 用例是否在 `src-core/src/application/`。
4. DTO 是否需要拆 UI 专用字段。
5. DTO 字段是否和 React 类型同步。
6. loading、error、snapshot 更新是否受影响。
