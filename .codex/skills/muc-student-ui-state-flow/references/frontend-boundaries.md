# 前端边界

- `src/main.tsx` 只负责挂载 React。
- `src/App.tsx` 只负责应用壳、页签、全局弹窗和状态分发。
- 业务逻辑优先落 `features`。
- 跨组件状态放 `stores`。
- React 订阅和复用 hook 放 `hooks`。
- 通用 UI 放 `components`。
- shadcn/ui 基础组件放 `components/ui`，不要放业务。
- 类型统一放 `types`。

# 常见反模式

- 在 `src/App.tsx` 里直接写一大坨调用和业务状态。
- 把数据整理、排序、过滤全塞进组件。
- 把业务行为塞进 `components/ui`。
- 前端字段已经变了，`types` 不跟。
