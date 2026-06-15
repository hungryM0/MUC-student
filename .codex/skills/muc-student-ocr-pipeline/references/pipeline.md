# OCR 链路

- `Backend::build` 创建原生 provider 和 worker provider
- `OcrProviderChain::new` 固定接收 `native, worker`
- `recognize_for_login` 先重试原生 3 次
- 原生失败后再调 worker
- 登录和自助面板都通过 OCR 链识别验证码

# 结果规范化

- `normalize_captcha_text`
过滤非字母数字字符，最多保留 4 位。

- `normalize_captcha_code`
在 parser 侧再做适配。
