---
name: muc-student-tauri-command-bridge
description: 处理 MUC-student 前端到 Rust 的 Tauri command、事件、DTO 和窗口行为桥接。仅在新增/改名 command、修改 `invoke` 参数或返回值、同步前后端 DTO、调整事件、或排查按钮点击到 Rust 用例链路时使用；纯 UI、纯 Rust service、纯 parser 改动不触发。
---

# MUC Student Tauri Command Bridge

先把整条桥接链找全。不要只改前端或只改 Rust 一头。

## 当前前端桥

先读 `src/lib/stores/app.ts`。这里集中管理：

- `invoke` command
- `listen` 事件
- `appSnapshot` 更新
- 任务开始和结束的 loading 文案
- 窗口关闭行为

## 当前 Rust 入口

先读：

- `src-tauri/src/adapters_tauri/`
- `src-tauri/src/application/backend.rs`
- `src-tauri/src/application/dto.rs`

## 改 command 的顺序

1. 找前端调用点。
2. 找 Tauri adapter 暴露点。
3. 找 `Backend` 里的实际用例。
4. 找 DTO 输入输出。
5. 找事件是否也要同步。

## DTO 规则

- 前端字段名看 `src/lib/types/app.ts`
- Rust 输入输出看 `application/dto.rs` 和 adapter 的反序列化结构
- 改字段时，前后端一起改
- 返回值如果是快照类对象，要检查 `isSnapshot` 判定还能不能成立

## 事件规则

当前有这些事件：

- `app://state-updated`
- `app://log-appended`
- `app://task-started`
- `app://task-finished`

改后台行为时，顺手检查这些事件是否还匹配前端预期。

## 常见坑

- 只改 `invoke` 名字，不改 Rust command 注册。
- Rust DTO 变了，前端 `types` 没变。
- 后端已经 emit 事件，前端没监听或字段没跟。
- 把业务直接塞进 adapter，桥接层变成屎山。

## 参考

- 桥接检查清单：`references/bridge-checklist.md`
- 当前 command 和事件地图：`references/current-bridge-map.md`
