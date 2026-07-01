# 覆盖率策略

## 工具

Rust 使用 `cargo-llvm-cov`。

仓库 alias：

- `cargo coverage-summary`
- `cargo coverage`

如果本机未安装：

```text
cargo install cargo-llvm-cov --locked
```

## 什么时候看

- 用户明确要求覆盖率。
- 刚补 parser、网络流程、迁移、自动切号等高风险测试。
- 修改共享服务后，需要确认旧测试仍打到关键路径。
- 发现测试数量增加但风险模块没有被执行。

## 怎么看

先跑摘要：

```text
cargo coverage-summary
```

摘要能回答“目标模块有没有被打到”。只有需要定位缺口时，再跑：

```text
cargo coverage
```

摘要模式会留下 `target/llvm-cov-target`。HTML 模式会输出到 `target/llvm-cov/html`。

看完删除这些目录：

```text
target/llvm-cov-target
target/llvm-cov
```

它们位于已忽略的 `target/` 下，不应出现在提交里。最终回复要说明覆盖率产物已清理。

## 不做

- 不把覆盖率阈值接入 CI，除非用户明确要求。
- 不因覆盖率数字低而扩大重构。
- 不写只为提高百分比的测试。
