# Slint 目录边界

- `src-slint/ui/app.slint` 是界面入口。
- `src-slint/ui/` 可以拆页面、组件和样式 token。
- `src-slint/src/main.rs` 绑定 callback，写入 global，调用后端。
- `src-slint/build.rs` 只编译 Slint 入口文件。
- `src-slint/target/` 是生成物。

# 禁区

- 不要恢复 React/Vite/npm 旧栈。
- 不要把业务流程塞进 `.slint`。
- 不要把后端 DTO 原样污染成界面模型。
- 不要把密码写进界面状态、日志或明文文件。
- 不要用解释性界面文案替代清晰控件。
