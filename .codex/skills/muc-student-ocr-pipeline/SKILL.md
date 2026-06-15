---
name: muc-student-ocr-pipeline
description: 处理这个仓库里的验证码识别链路、OCR provider 顺序、模型资源路径和失败回退逻辑。用在改 `infrastructure/ocr`、认证验证码登录、OCR worker、资源加载，或排查原生 OCR 与 worker 兜底行为的时候。
---

# MUC Student OCR Pipeline

这条链路有硬顺序。不要自作聪明改成别的。

## 固定顺序

`NativeRustOcrProvider` -> `ExternalWorkerOcrProvider`

在 `Backend::build` 里先构造原生 provider，再构造 worker，再交给 `OcrProviderChain::new`。

## 当前行为

看 `src-tauri/src/infrastructure/ocr/provider.rs`：

- 原生 OCR 最多尝试 3 次
- 返回非空结果就直接用
- 原生连续失败后，才走 worker
- worker 也返回空时，报 OCR 错误
- 两边都失败时，报双 provider 失败

## 改动规则

1. 改 provider 行为前，先读 `provider.rs`、`native_rust_provider.rs`、`external_worker_provider.rs`。
2. 改资源路径前，先读 `RuntimePaths` 和 `Backend::build`。
3. 改登录验证码识别时，同时检查 `AuthPortalClient` 和 `SelfServicePanelClient`。
4. 不要只改一处的验证码规范化逻辑。

## 不准动歪的地方

- 不要把 worker 提前到原生前面。
- 不要把 OCR 资源路径硬编码进业务逻辑。
- 不要在 parser 里塞 OCR 调用。
- 不要把验证码失败重试散落到多个层里。

## 排查顺序

1. 资源路径是否正确。
2. 原生 OCR 是否返回空串或脏字符。
3. `normalize_captcha_text` 或 `normalize_captcha_code` 是否截断过头。
4. 客户端重试次数和错误分支是否符合预期。
5. worker 是否只在兜底时触发。

## 参考

- 链路说明：`references/pipeline.md`
- 资源和调用点：`references/paths-and-callers.md`
