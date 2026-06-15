# 验证矩阵

## 前端应用壳、feature、组件、hook、store、type

- `npm run check`
- 影响构建、React 入口、Tailwind/shadcn 配置或静态资源时加 `npm run build`

## Rust 领域、应用、基础设施

- `cargo check`
- 有相关测试就跑 `cargo test`

## Tauri command、DTO、事件桥接

- `npm run check`
- `cargo check`
- 必要时 `cargo test`

## Tauri 打包、托盘、自启、Windows 行为

- `npm run check`
- `cargo check`
- `cargo test`
- `npm run tauri -- build`
