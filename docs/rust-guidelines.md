# Rust 编码规范

## 1. 错误处理

- 在应用层和基础设施层使用 `anyhow::Result<T>`
- 对于可复用、有类型的错误枚举使用 `thiserror`
- 生产代码中禁止使用 `unwrap()` — 使用 `?` 或 `.unwrap_or_else`
- 传播错误时附加上下文：`.map_err(|e| anyhow::anyhow!("context: {e}"))`

## 2. 异步编程

- 使用 `async/await`，trait 方法通过 `#[async_trait]` 实现
- 除 `main()` 外禁止使用 `block_on()`
- 共享状态使用 `Arc<dyn Trait>` 进行依赖注入

## 3. 代码风格

- 函数应小而专一，职责单一
- 避免深层嵌套 — 使用提前返回（early return）
- 优先使用模式匹配，而非链式 `if let`
- 命名不使用缩写，保持清晰描述性

## 4. 日志记录

- 使用 `tracing::info/warn/error` 宏记录日志
- 禁止在运行时使用 `println!` 输出（仅限 CLI 用户可见输出）
- 结构化字段格式：`tracing::info!(key = %value, "message")`

## 5. Trait 定义示例

```rust
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    async fn fetch_logs(&self, range: &QueryRange) -> anyhow::Result<Vec<LogEntry>>;
}
```
