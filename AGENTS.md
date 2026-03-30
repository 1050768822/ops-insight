# AGENTS.md — ops-insight

本文件面向 AI 编码代理（OpenAI Codex、GitHub Copilot 等），提供在此代码库中高效工作所需的全部信息。

---

## 项目概览

ops-insight 是一个 Rust CLI 工具，用于读取运维日志（来自 New Relic API 或本地 Serilog 文件），通过 AI（Claude 或 OpenAI）进行分析，并生成运维报告。

- 语言：Rust edition 2024
- 异步运行时：Tokio

---

## 构建与运行

```bash
cargo build -p ops-insight-core                          # 编译 CLI
cargo run -p ops-insight-core -- config init             # 生成 config.toml 模板
cargo run -p ops-insight-core -- serilog --path ./logs   # 分析本地日志
cargo run -p ops-insight-core -- daily                   # 昨日报告（需要 New Relic）
cargo run -p ops-insight-core -- weekly                  # 本周报告
cargo run -p ops-insight-core -- custom --from 2026-03-01 --to 2026-03-07
cargo tauri dev                                          # 启动桌面 GUI（需安装 tauri-cli）
```

---

## 规范文档（必读）

修改代码前必须阅读以下文档：

- `docs/clean-architecture.md` — 架构层级与依赖规则
- `docs/rust-guidelines.md` — Rust 编码规范
- `docs/security.md` — API Key 安全规则
- `docs/log-formats.md` — 日志格式与解析规则

---

## 项目结构

```
ops-insight/                        # Cargo workspace 根目录
├── Cargo.toml                      # workspace manifest
├── ops-insight-core/               # 共享库 + CLI 二进制
│   ├── Cargo.toml
│   ├── config.example.toml
│   └── src/
│       ├── main.rs                 # CLI 入口：配置加载、依赖注入、命令分发
│       ├── lib.rs                  # 公开 API（供 Tauri 使用）
│       ├── config.rs               # Config* 结构体
│       ├── factory.rs              # build_analyzer(), load_config()
│       ├── helpers.rs              # parse_custom_range(), serilog_range()
│       ├── domain/
│       │   ├── entities/           # 核心数据结构（LogEntry, ErrorEvent, Report 等）
│       │   ├── ports/              # Trait 定义（DataSource, Analyzer, ReportWriter）
│       │   └── value_objects/      # 值对象（SecretKey）
│       ├── application/
│       │   ├── dtos/               # Use Case 的输入/输出结构
│       │   └── use_cases/          # 业务编排（GenerateReportUseCase）
│       ├── infrastructure/
│       │   ├── newrelic/           # DataSource impl — New Relic NerdGraph API
│       │   ├── serilog/            # DataSource impl — 本地 Serilog 日志文件
│       │   ├── claude/             # Analyzer impl — Claude API
│       │   ├── openai/             # Analyzer impl — OpenAI API
│       │   └── output/             # ReportWriter impl — terminal + markdown
│       └── interfaces/
│           └── cli/                # clap CLI 命令定义
├── src-tauri/                      # Tauri 2.0 桌面应用
└── frontend/                       # 桌面 GUI（HTML/CSS/JS）
```

---

## 扩展模式

项目通过三个核心 trait 支持扩展，新增实现时只需在 `infrastructure` 层添加代码，其他层无需改动。

**新增数据源** — 在 `ops-insight-core/src/infrastructure/xxx/source.rs` 中实现 `DataSource` trait

**新增分析器** — 在 `ops-insight-core/src/infrastructure/xxx/analyzer.rs` 中实现 `Analyzer` trait

**新增输出方式** — 在 `ops-insight-core/src/infrastructure/output/xxx.rs` 中实现 `ReportWriter` trait

**接入方式** — 在 `ops-insight-core/src/main.rs` 中完成依赖注入，其他层不需要改动

---

## API Key 规则（必须遵守）

- 禁止将 API Key 存储为裸 `String`，必须使用 `SecretKey` 值对象包装
- 只能在 `use_key("purpose", |k| ...)` 闭包内访问 Key 的实际值
- 每次访问会自动记录审计日志

正确示例：

```rust
self.api_key.use_key("openai_request", |key| {
    client.header("Authorization", format!("Bearer {key}")).send()
})
```

---

## 禁止事项

- Domain 层禁止导入 `reqwest`、数据库 crate 等基础设施依赖
- Interfaces 层禁止直接调用 Infrastructure 层
- 禁止使用裸 `String` 传递 API Key
- 禁止使用 `unwrap()`，使用 `?` 或 `unwrap_or_else` 替代
- 禁止提交 `config.toml`（已在 `.gitignore` 中排除）

---

## 新增分析器步骤

1. 在 `ops-insight-core/src/infrastructure/xxx/analyzer.rs` 中实现 `Analyzer` trait
2. 在 `ops-insight-core/src/infrastructure/xxx/mod.rs` 中导出该实现
3. 在 `ops-insight-core/src/infrastructure/mod.rs` 中添加模块声明
4. 在 `ops-insight-core/config.example.toml` 的 `[analyzer]` 中添加对应的 provider 名称
5. 在 `ops-insight-core/src/factory.rs` 的 `build_analyzer()` 函数中添加对应分支
