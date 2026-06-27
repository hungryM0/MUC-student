---
name: muc-student-architecture
description: 规划或审查 MUC-student 仓库的代码边界、模块落点和分层约束。仅在跨 React 界面、Tauri/Rust 业务核心多层改动、目录迁移、服务拆分、领域层边界审查、或改动位置不确定时使用；小范围 bugfix、单文件改名、纯测试运行不触发。
---

# MUC Student Architecture

先读仓库内的 `AGENTS.md`，再读这次改动会碰到的入口文件。不要看文件名瞎写。

## 先做什么

先列出这次改动会碰到的层和入口：

- React 界面入口：`src/main.tsx`
- React 页面：`src/pages/`
- React 组件：`src/components/`
- 前端 Tauri invoke 封装和 DTO 类型：`src/lib/`
- Tauri command 和插件注册：`src-tauri/src/lib.rs`
- Tauri 桌面平台适配：`src-tauri/src/platform.rs`
- Tauri 插件封装：`src-tauri/src/plugins/`
- Rust 用例、服务、DTO 和运行时编排：`src-core/src/application/`
- Rust 领域模型和策略：`src-core/src/domain/`
- Rust 外部系统、持久化、网络、解析、安全、系统适配：`src-core/src/infrastructure/`

先给出改动落点，再改文件。

## 落点规则

1. 这是 React 页面、组件、布局和视觉风格。
放 `src/pages/` 或 `src/components/`。

2. 这是前端 DTO、Tauri invoke 封装、窗口工具。
放 `src/lib/`。

3. 这是 Tauri command 暴露、插件注册、WebView 桌面壳装配。
放 `src-tauri/src/lib.rs` 或 `src-tauri/src/plugins/`。

4. 这是 Windows 路径、自启、平台 API。
放 `src-tauri/src/platform.rs`。

5. 这是用例编排、流程控制、DTO 组装。
放 `src-core/src/application/`。

6. 这是纯业务规则、领域模型、选择策略、计算逻辑。
放 `src-core/src/domain/`。这里别塞网络请求、文件读写、系统调用。

7. 这是 HTTP、文件、凭据、系统、自启、解析器、持久化。
放 `src-core/src/infrastructure/`。

## 硬规则

- 不要手改 `build/`、`dist/`、`target/`、`src-tauri/target/`。
- 不要把业务网络、持久化、凭据库或账号选择策略塞进 React。
- 不要把密码写回 JSON。凭据继续走 `credential_vault`。
- 改本地存储格式时，先补 `src-core/src/infrastructure/persistence/migration.rs`。
- `domain` 保持纯。看到网络请求、文件读写、系统 API 掉进 `domain`，直接当成脏活处理掉。

## 界面到后端怎么查

先找 React 入口，再顺着调用链往下读：

`src/pages/*`
-> `src/lib/muc.ts`
-> `src-tauri/src/lib.rs`
-> `src-core/src/application/`
-> `src-core/src/infrastructure/` / `src-core/src/domain/`

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
