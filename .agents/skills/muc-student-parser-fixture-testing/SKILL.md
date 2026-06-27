---
name: muc-student-parser-fixture-testing
description: 规范 MUC-student 的 HTML parser fixture、脱敏样本和回归测试。仅在新增或修改 `src-core/src/infrastructure/parsers/`、`src-core/tests/fixtures/`、HTML 样本、parser 回归测试，或用户要求补 parser 测试样本时使用；纯业务流程测试、纯前端测试不触发。
---

# MUC Student Parser Fixture Testing

parser 测试要固定真实页面结构带来的风险，不要只测手写玩具 HTML。

## 落点

- parser 代码：`src-core/src/infrastructure/parsers/`。
- 小样本单测：可以放在 parser 文件内的 `#[cfg(test)]`。
- 复用 HTML fixture：放在 `src-core/tests/fixtures/parsers/`。
- 跨 parser 或链路级样本：放 integration test，不塞进 parser 职责。

## Fixture 规则

- 先从真实响应裁剪，再脱敏；不要凭想象重写 DOM。
- 保留 parser 依赖的标签层级、表单字段、meta、script 片段、链接路径和编码特征。
- 删除无关样式、广告、脚本体和大段静态资源。
- 账号、姓名、手机号、IP、cookie、token、学校内部域名和 session id 必须脱敏。
- 脱敏值要稳定，例如 `2024000000`、`user_a`、`192.0.2.10`、`https://portal.example.test/`。
- fixture 文件名用页面和场景命名，例如 `portal-success-online.html`、`panel-home-with-devices.html`。

## 测试要求

- 每个 fixture 至少有一个正向断言和一个关键字段断言。
- parser 修改必须补对应回归测试，覆盖曾经失败的 HTML 形态。
- 不要只断言 `is_some()`；要断言账号、IP、流量、csrf、下线链接、SSO URL 等稳定输出。
- 对兼容分支分别命名测试，例如 meta fallback、相对 URL、缺失字段、未知本机 IP。
- parser 测试不能发网络请求，不能依赖本机 IP，不能读用户真实配置。

## 检查

- 改 parser 或 fixture 后跑 `cargo test`。
- 大量 fixture 调整后可跑 `cargo coverage-summary` 看 parser 模块是否被测试实际打到。
- 覆盖率报告只看结果，不提交或保留 `target/llvm-cov-target`、`target/llvm-cov/`。

## 参考

- Fixture 细则：`references/parser-fixtures.md`
