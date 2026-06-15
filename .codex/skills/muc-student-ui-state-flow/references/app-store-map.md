# app.ts 现有能力

- 应用启动：`bootstrapApp`
- 初始化桥接：`initializeTauriBridge`
- 订阅 store：`subscribeAppStore`
- 读取 store 快照：`getAppStoreSnapshot`
- 拉后端快照：`getBackendSnapshot`
- 账号操作：`selectAccount`、`createAccount`、`updateAccount`、`deleteAccount`
- 状态操作：`loginSelectedAccount`、`refreshDashboard`、`logoutLocalDevice`
- 设置操作：`updatePreferences`
- 对话框状态：`open*`、`closeDialog`
- 全局 UI 状态：`setActivePage`、`setSortMode`、`clearError`
