# ops-insight — 开发规则

## 项目定位

Rust CLI 工具，读取运维日志（New Relic API 或本地 Serilog 文件），通过 Claude API 智能分析，生成运维报告。

## Language & Runtime

- Language: **Rust** (edition 2024)
- Async runtime: **Tokio**
- Package manager: **Cargo**

## 规范文档

编写代码前必须阅读对应文档：

- 架构规则：`docs/clean-architecture.md`
- Rust 规范：`docs/rust-guidelines.md`
- 安全规则：`docs/security.md`
- 日志格式：`docs/log-formats.md`

---

## 禁止事项

- 不得在 Domain 层导入 `reqwest`、数据库 crate、HTTP 类型
- 不得在 Interfaces 层直接调用 Infrastructure（必须通过 Use Case）
- 不得将 API Key 以 `String` 类型传递或存储，必须使用 `SecretKey`
- 不得在 `config.toml` 中提交真实 Key（`.gitignore` 已忽略）

---

## 新增命令流程

1. `src/interfaces/cli/commands.rs` — 添加 `Command` 枚举变体
2. `src/main.rs` — 在 `match` 中注入合适的 DataSource / Analyzer / Writers
3. 如需新 Use Case → `src/application/use_cases/` 新建文件

## 配置文件

- `config.example.toml`：模板，提交到 git
- `config.toml`：实际配置，不提交（已 `.gitignore`）
- 生成配置：`ops-insight config init`
