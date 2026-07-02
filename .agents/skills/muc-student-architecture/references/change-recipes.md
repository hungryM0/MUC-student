# 常见改动落点

## 新增 React 功能

- 页面、布局、局部状态：`src/pages/`。
- 复用 UI：`src/components/`。
- hook：`src/hooks/`。
- invoke 封装和前端 DTO：`src/lib/`。
- 后端能力缺口：先补 `src-core/src/application/` 用例，再接 Tauri command。

## 新增后端用例

- Tauri command：`src-tauri/src/lib.rs`。
- 用例编排：`src-core/src/application/`。
- 纯规则：`src-core/src/domain/`。
- 外部系统：`src-core/src/infrastructure/`。
- 前端入口：`src/lib/muc.ts`。

## 改存储结构

1. 先确认模型和 DTO。
2. 再改 repository。
3. 补 `src-core/src/infrastructure/persistence/migration.rs`。
4. 跑读取、写回和迁移相关测试。

## 改网络链路

1. 从 `src-core/src/application/backend.rs` 或 service 入口读起。
2. 再看 infrastructure client。
3. 最后改 parser。
4. 不要把流程判断塞进 parser。
