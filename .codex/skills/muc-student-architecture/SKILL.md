---
name: muc-student-architecture
description: 规划或审查 MUC-student 仓库的代码边界、模块落点和分层约束。仅在跨 Slint 界面、遗留 Tauri/Rust 业务核心多层改动、目录迁移、服务拆分、领域层边界审查、或改动位置不确定时使用；小范围 bugfix、单文件改名、纯测试运行不触发。
---

# MUC Student Architecture

先读仓库内的 `AGENTS.md`，再读这次改动会碰到的入口文件。不要看文件名瞎写。

## 先做什么

先列出这次改动会碰到的层和入口：

- Slint 界面入口：`src-slint/ui/app.slint`
- Slint Rust 装配：`src-slint/src/main.rs`
- Slint 构建脚本：`src-slint/build.rs`
- 遗留 Tauri 适配层：`src-tauri/src/adapters_tauri/`
- Rust 用例和运行时编排：`src-tauri/src/application/`
- Rust 领域模型和策略：`src-tauri/src/domain/`
- Rust 外部系统、持久化、网络、解析、安全、系统适配：`src-tauri/src/infrastructure/`

先给出改动落点，再改文件。

## 落点规则

1. 这是 Slint 界面结构、控件、布局、视觉风格。
放 `src-slint/ui/`。

2. 这是 Slint 窗口创建、global 写入、callback 绑定。
放 `src-slint/src/main.rs`。

3. 这是 Slint 编译入口。
放 `src-slint/build.rs`。这里只调用 `slint_build::compile(...)`。

4. 这是遗留 Tauri command 暴露、事件桥接、WebView 桌面壳适配。
放 `src-tauri/src/adapters_tauri/`。

5. 这是用例编排、流程控制、DTO 组装。
放 `src-tauri/src/application/`。

6. 这是纯业务规则、领域模型、选择策略、计算逻辑。
放 `src-tauri/src/domain/`。这里别塞网络请求、文件读写、系统调用。

7. 这是 HTTP、文件、凭据、系统、自启、解析器、持久化。
放 `src-tauri/src/infrastructure/`。

## 硬规则

- 不要手改 `build/`、`src-tauri/gen/`。
- 不要手改 `src-slint/target/`。
- 不要恢复 React/Vite/npm 旧栈。
- 不要把密码写回 JSON。凭据继续走 `credential_vault`。
- 改本地存储格式时，先补 `src-tauri/src/infrastructure/persistence/migration.rs`。
- `domain` 保持纯。看到网络请求、文件读写、系统 API 掉进 `domain`，直接当成脏活处理掉。

## 界面到后端怎么查

先找 Slint 入口，再顺着调用链往下读：

`src-slint/ui/app.slint`
-> `src-slint/src/main.rs`
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
