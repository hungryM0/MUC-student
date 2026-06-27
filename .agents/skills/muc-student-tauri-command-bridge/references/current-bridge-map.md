# Command

- `bootstrapApp`
- `getAppSnapshot`
- `selectAccount`
- `addAccount`
- `updateAccount`
- `deleteAccount`
- `loginSelectedAccount`
- `refreshDashboard`
- `logoutLocalDevice`
- `updatePreferences`
- `updateTrayMenu`

# 事件

- `muc://state-updated`
- `muc://task-started`
- `muc://task-finished`

# 关键落点

- React 入口：`src/main.tsx`
- React invoke 封装：`src/lib/muc.ts`
- Tauri command：`src-tauri/src/lib.rs`
- 托盘插件：`src-tauri/src/plugins/system_tray.rs`
- 后端核心：`src-core/src/application/backend.rs`
- DTO：`src-core/src/application/dto.rs`
