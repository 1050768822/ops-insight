# API Key 安全规范

## 1. SecretKey 包装类型

位置：`ops-insight-core/src/domain/value_objects/secret_key.rs`

- 将密钥存储在私有字段中，外部不可直接访问
- `Debug`/`Display` 输出 `[REDACTED]` — 永远不泄露实际值
- 实现 `ZeroizeOnDrop` — 变量被销毁时自动清零内存
- 仅通过 `use_key(purpose, closure)` 暴露密钥，每次访问自动记录审计日志

## 2. 使用规则

- 所有 API Key 必须使用 `SecretKey` 包装，禁止使用原始 `String`
- 在 HTTP 请求中必须通过 `use_key()` 访问密钥
- 每次访问通过 `tracing::info!(key_label, purpose, "api_key_accessed")` 自动记录审计日志

## 3. 正确与错误示例

```rust
// ✅ 正确
let key = SecretKey::new("newrelic_api_key", raw_string);
key.use_key("newrelic_nrql_request", |k| {
    client.header("Api-Key", k).send()
});

// ❌ 错误 — 原始字符串直接存储和传递
struct Foo { api_key: String }
client.header("Api-Key", &self.api_key)
```

## 4. 密钥来源优先级（从高到低）

1. macOS Keychain（`security find-generic-password`）
2. 环境变量（`NEWRELIC_API_KEY`、`CLAUDE_API_KEY`）
3. `config.toml` 配置文件

## 5. 将密钥存入 Keychain

```bash
security add-generic-password -a "$USER" -s "newrelic_api_key" -w "NRAK-xxx"
security add-generic-password -a "$USER" -s "claude_api_key" -w "sk-ant-xxx"
```
