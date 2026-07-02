# Parser Fixture 细则

## 采样流程

1. 保存真实响应时先放在临时目录，不要提交原始文件。
2. 裁剪到 parser 需要的最小 DOM。
3. 脱敏账号、IP、cookie、token、姓名、手机号、校内域名和时间戳。
4. 用测试确认裁剪后的 fixture 仍能复现目标输出。
5. 删除原始样本和临时覆盖率报告。

## 推荐目录

```text
src-core/tests/fixtures/parsers/
  portal-success-online.html
  portal-login-form.html
  panel-home-with-devices.html
  panel-home-no-local-device.html
  online-devices-relative-logout.html
```

小样本可以直接写在 parser 单测里。超过约 30 行、要复用、或来自真实页面裁剪时，放 fixture 文件。

## 脱敏规则

- 用户名：`2024000000`、`2024000001`。
- 账号别名：`user_a`、`user_b`。
- IPv4：公网示例网段用 `192.0.2.10`、`198.51.100.10`。
- 域名：`portal.example.test`、`panel.example.test`。
- cookie/session：`SESSION_TEST_VALUE`。
- csrf/token：`csrf_test_value`。
- 流量和余额可以保留量级，但不要保留真实账单信息。

## 断言质量

好的 parser 回归测试要断言业务字段，不要只断言解析成功。

- portal 成功页：账号、本机 IP、已用流量、计费模式。
- portal 登录页：隐藏字段、`ac_id` fallback、meta refresh 目标。
- panel home：套餐名、总量、已用量、csrf、本机设备匹配结果。
- online devices：设备 IP、下线 URL、绝对和相对路径归一化。

## 禁止

- 禁止提交未脱敏的真实 HTML。
- 禁止把 fixture 当业务配置读。
- 禁止 parser 测试访问网络。
- 禁止为了覆盖率写无意义断言。
