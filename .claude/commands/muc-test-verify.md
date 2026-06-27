为 MUC-student 改动选择最小验证命令，并说明已跑、未跑和失败原因。

按改动类型选最小检查，不要用一堆命令掩盖没看懂问题。

## 验证矩阵

**只改 React 页面、组件、hook、样式**
```
pnpm build
pnpm lint
```

**只改 Rust 业务核心**
```
cargo fmt --check
cargo check
```
改了策略、解析、持久化、网络流程时再跑：
```
cargo test
```

**改 React 和 Rust/Tauri 对接**
```
pnpm build
pnpm lint
cargo fmt --check
cargo check
```
改了 DTO、command、事件、持久化或迁移时再跑：
```
cargo test
```

**改 Tauri、托盘、自启、Windows 行为、发布配置**
```
cargo fmt --check
cargo check
cargo test
```
发布前按需加：
```
pnpm build
cargo build --release
```

## 输出规则

- 明确列出已跑命令和结果。
- 没跑的命令要说明原因。
- 失败时只先修相关失败，不顺手扩大范围。
- 不创建 md 验证清单文档。
- 测试覆盖率或临时报告文件用完删除。
