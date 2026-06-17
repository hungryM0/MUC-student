---
name: muc-student-architecture
description: 规划或审查 MUC-student 仓库的代码边界、模块落点和分层约束。仅在跨前端/Tauri/Rust 多层改动、目录迁移、服务拆分、领域层边界审查、或改动位置不确定时使用；小范围 bugfix、单文件改名、纯测试运行不触发。
---

# MUC Student Architecture

先读仓库内的 `AGENTS.md`，再读这次改动会碰到的入口文件。不要看文件名瞎写。

## 先做什么

先列出这次改动会碰到的层和入口：

- React 挂载入口：`src/main.tsx`
- React 应用壳：`src/App.tsx`
- shadcn/ui 基础组件：`src/lib/components/ui/`
- 前端通用组件：`src/lib/components/`
- 前端功能逻辑：`src/lib/features/`
- 前端 hooks：`src/lib/hooks/`
- 前端状态：`src/lib/stores/`
- 前端类型：`src/lib/types/`
- Tauri 适配层：`src-tauri/src/adapters_tauri/`
- Rust 用例和运行时编排：`src-tauri/src/application/`
- Rust 领域模型和策略：`src-tauri/src/domain/`
- Rust 外部系统、持久化、网络、解析、安全、系统适配：`src-tauri/src/infrastructure/`

先给出改动落点，再改文件。

## 落点规则

按这个顺序判断：

1. 这是 React 挂载入口。
放 `src/main.tsx`。这里只挂载根组件和引入全局样式。

2. 这是应用壳、全局布局、页签切换或全局弹窗装配。
放 `src/App.tsx`。这里不要堆业务流程。

3. 这是 shadcn/ui 基础组件。
放 `src/lib/components/ui/`。这里只放 UI 原语，不放业务。

4. 这是可复用界面片段。
放 `src/lib/components/`。只放显示层和轻交互。

5. 这是某个页面或功能的业务逻辑。
优先放 `src/lib/features/<feature>/`。不要继续把 `components` 写成垃圾场。

6. 这是 React hooks。
放 `src/lib/hooks/`。

7. 这是跨组件共享的前端状态。
放 `src/lib/stores/`。

8. 这是前后端传输类型或前端模型。
放 `src/lib/types/`。

9. 这是 Tauri command 暴露、事件桥接、桌面壳适配。
放 `src-tauri/src/adapters_tauri/`。

10. 这是用例编排、流程控制、DTO 组装。
放 `src-tauri/src/application/`。

11. 这是纯业务规则、领域模型、选择策略、计算逻辑。
放 `src-tauri/src/domain/`。这里别塞网络请求、文件读写、系统调用。

12. 这是 HTTP、文件、凭据、系统、自启、解析器、持久化。
放 `src-tauri/src/infrastructure/`。

## 硬规则

- 不要手改 `build/`、`src-tauri/gen/`。
- 不要把密码写回 JSON。凭据继续走 `credential_vault`。
- 改本地存储格式时，先补 `src-tauri/src/infrastructure/persistence/migration.rs`。
- `domain` 保持纯。看到网络请求、文件读写、系统 API 掉进 `domain`，直接当成脏活处理掉。

## 前后端联动时怎么查

先找前端入口，再顺着桥接链往下读：

`src/App.tsx` / `src/lib/features` / `src/lib/components`
-> `src/lib/hooks` / `src/lib/stores`
-> Tauri `invoke` / 事件监听
-> `src-tauri/src/adapters_tauri/`
-> `src-tauri/src/application/`
-> `src-tauri/src/infrastructure/` / `src-tauri/src/domain/`

如果改动跨过 3 层以上，先写一句链路说明，再下手。

## 大改处理法

大改前先回答四件事：

1. 目标是什么。
2. 入口在哪。
3. 新逻辑属于哪一层。
4. 哪些旧模块不能被顺手塞脏东西。

如果答不清，先补阅读，不要赌。

## 参考

- 目录边界和禁区：`references/boundaries.md`
- 常见改动怎么落点：`references/change-recipes.md`
