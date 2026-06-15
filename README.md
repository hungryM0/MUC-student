# MUC-student

适用于 MUC 校园网多账号拼车的桌面应用，节省网费

## 功能

- 多账号添加、编辑、删除、选择登录。
- 校园网 HTTP 认证。
- 自助面板状态刷新、在线设备查询、本机下线。
- 流量配额汇总和账号配额展示。
- 流量用尽后按最近使用优先策略自动切号。
- 关闭到托盘、托盘恢复、开机自启。
- 旧版本地数据首次启动自动迁移。
- 密码写入 Windows Credential Manager，本地 JSON 不保存明文密码。

## 本地开发

```bash
npm install
npm run tauri -- dev
```

## 检查

```bash
npm run check
npm run build
cd src-tauri
cargo check
cargo test
```

## 打包

```bash
npm run tauri -- build
```

打包产物在：

- `src-tauri/target/release/bundle/msi/`
- `src-tauri/target/release/bundle/nsis/`

## OCR

OCR 按 provider 链执行：

1. `NativeRustOcrProvider`
2. `ExternalWorkerOcrProvider`

原生模型和 worker 放在 `src-tauri/resources/ocr/`。
