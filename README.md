# MUC-student

适用于 MUC 校园网多账号拼车的桌面应用。

![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-0078D4?logo=windows&logoColor=white)

> [!NOTE]
>
> 这是一个前端学习的个人练手项目。
>
> 仅供学习交流，**请不要将账号密码告诉不信任的人！**

## 功能

- 多账号添加、编辑、选择登录
- 校园网自动认证、自动重试登录
- 流量配额汇总和账号配额展示
- 流量用尽后自动切号

## 开发

```powershell
pnpm install
pnpm tauri:dev
```

核心业务在 `src-core/`。Tauri 桌面壳在 `src-tauri/`。React 前端在 `src/`。

常用检查：

```powershell
pnpm build
cargo fmt --check
cargo check
cargo test
```
