# 遗留 command

- `bootstrapApp`
- `getAppSnapshot`
- `selectAccount`
- `createAccount`
- `updateAccount`
- `deleteAccount`
- `loginSelectedAccount`
- `refreshDashboard`
- `logoutLocalDevice`
- `updatePreferences`

# 遗留事件

- `app://state-updated`
- `app://log-appended`
- `app://task-started`
- `app://task-finished`

# Rust 关键落点

- 旧 adapter：`src-tauri/src/adapters_tauri/`
- 旧 Tauri 壳：`src-tauri/src/lib.rs`
- backend：`src-tauri/src/application/backend.rs`
- DTO：`src-tauri/src/application/dto.rs`
- 新 Slint 装配：`src-slint/src/main.rs`
