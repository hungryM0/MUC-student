# MUC-student

适用于 MUC 校园网多账号拼车的桌面应用，节省网费

![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![WinUI 3](https://img.shields.io/badge/WinUI%203-0078D4?logo=windows&logoColor=white)
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

当前是 Rust workspace：

```powershell
cargo run
```

核心业务在 `src-core/`，Windows 原生界面入口在 `src-winui/`。
