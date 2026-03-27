# ops-insight

New Relic 运维报告 CLI 工具，支持从 New Relic 拉取服务器日志与错误数据，或读取本地 Serilog 日志文件，通过 Claude / OpenAI / 本地规则引擎进行智能分析，自动生成运维报告。

---

## 快速开始

### 1. 安装

```bash
git clone https://github.com/yourname/ops-insight
cd ops-insight
cargo build --release
# 可选：安装到系统路径
cargo install --path .
```

### 2. 初始化配置

```bash
ops-insight config init
```

这会在当前目录生成 `config.toml`，填写你的 API Key 和服务器列表即可。

### 3. 生成报告

```bash
# 生成昨日报告
ops-insight daily

# 生成上周报告
ops-insight weekly

# 自定义时间范围
ops-insight custom --from 2026-03-01 --to 2026-03-07

# 分析本地 Serilog 日志（文件或目录）
ops-insight serilog --path ./logs/
ops-insight serilog --path ./logs/log_20260327.log
ops-insight serilog --path ./logs/ --from 2026-03-20 --to 2026-03-27
```

---

## 配置文件

配置文件默认路径为当前目录下的 `config.toml`，可用 `--config` 指定其他路径。

```toml
[newrelic]
api_key = "NRAK-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
account_id = "1234567"

[analyzer]
# 选择分析引擎: "claude" | "openai" | "local"
provider = "local"

[claude]
api_key = "sk-ant-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
model = "claude-opus-4-6"

[openai]
api_key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
model = "gpt-4o"

[output]
# 输出格式: "terminal" | "markdown" | "both"
format = "both"
output_dir = "./reports"
# 报告语言: "zh"（中文，默认）| "en"（English）
language = "zh"

# 监控的服务器列表（留空则查询所有）
[[servers]]
name = "web-01"
hostname = "web-01.example.com"

[[servers]]
name = "worker-01"
hostname = "worker-01.example.com"
```

---

## CLI 命令

| 命令 | 说明 |
|------|------|
| `ops-insight daily` | 生成昨日运维报告（New Relic） |
| `ops-insight weekly` | 生成上周运维报告（New Relic） |
| `ops-insight custom --from YYYY-MM-DD --to YYYY-MM-DD` | 自定义时间范围报告（New Relic） |
| `ops-insight serilog --path <路径>` | 分析本地 Serilog 日志文件或目录 |
| `ops-insight serilog --path <路径> --from <日期> --to <日期>` | 指定时间范围分析 Serilog 日志 |
| `ops-insight config init` | 生成 config.toml 配置模板 |
| `ops-insight --config /path/to/config.toml daily` | 指定配置文件路径 |

---

## 分析引擎对比

| 引擎 | 配置 | 特点 |
|------|------|------|
| `claude` | 需要 Claude API Key | 深度语义分析，中英文报告质量最高 |
| `openai` | 需要 OpenAI API Key | GPT-4o 分析，可作为 Claude 替代 |
| `local` | 无需 API Key | 本地规则引擎，支持离线分析，适合敏感环境 |

本地分析引擎（`local`）内置规则：

- **敏感数据检测**：扫描日志中是否泄漏 API Key、密码、Token、邮箱、数据库连接字符串等
- **接口统计**：统计请求最多的接口、慢接口（>1000ms）、高错误率接口（>10%）

---

## API Key 安全

API Key 优先级（由高到低）：

1. **macOS Keychain**（推荐）
2. **环境变量**
3. **config.toml 配置文件**

### 使用 Keychain 存储（推荐）

```bash
# 存储 New Relic API Key
security add-generic-password -a $USER -s newrelic_api_key -w "NRAK-xxx"

# 存储 Claude API Key
security add-generic-password -a $USER -s claude_api_key -w "sk-ant-xxx"

# 存储 OpenAI API Key
security add-generic-password -a $USER -s openai_api_key -w "sk-xxx"
```

使用 Keychain 后，config.toml 中的 api_key 字段可留空。

### 使用环境变量

```bash
export NEWRELIC_API_KEY="NRAK-xxx"
export CLAUDE_API_KEY="sk-ant-xxx"
export OPENAI_API_KEY="sk-xxx"
```

> API Key 在内存中使用 `SecretKey` 包装，打印时显示 `[REDACTED]`，程序退出时自动清零（ZeroizeOnDrop），每次访问均记录审计日志。

---

## 架构说明

本项目采用 Clean Architecture，依赖方向由外向内单向流动：

```
interfaces (CLI)
    ↓
application (Use Cases)
    ↓
domain (Entities, Traits)
    ↑
infrastructure (New Relic, Claude, OpenAI, Local, Output)
```

### 核心扩展点

| Trait | 职责 | 内置实现 |
|-------|------|----------|
| `DataSource` | 数据来源 | `NewRelicSource`, `SerilogFileSource` |
| `Analyzer` | 智能分析 | `ClaudeAnalyzer`, `OpenAiAnalyzer`, `LocalAnalyzer` |
| `ReportWriter` | 报告输出 | `TerminalWriter`, `MarkdownWriter` |
| `LocalRule` | 本地分析规则 | `SensitiveDataRule`, `EndpointStatsRule` |

### 项目结构

```
ops-insight/
├── Cargo.toml
├── config.example.toml
├── CLAUDE.md                    # AI 编码助手规则
├── AGENTS.md                    # AI Agent 规则
├── docs/
│   ├── clean-architecture.md
│   ├── rust-guidelines.md
│   ├── security.md
│   └── log-formats.md
└── src/
    ├── main.rs
    ├── domain/
    │   ├── entities/
    │   │   ├── log_entry.rs     # LogEntry, LogLevel, ErrorEvent
    │   │   └── report.rs        # Report, Issue, Suggestion, Severity
    │   ├── ports/
    │   │   ├── data_source.rs   # trait DataSource
    │   │   ├── analyzer.rs      # trait Analyzer, AnalysisInput/Output
    │   │   └── report_writer.rs # trait ReportWriter
    │   └── value_objects/
    │       └── secret_key.rs    # SecretKey（ZeroizeOnDrop + 审计日志）
    ├── application/
    │   ├── dtos/                # QueryRange, ReportDto
    │   └── use_cases/
    │       └── generate_report.rs
    ├── infrastructure/
    │   ├── newrelic/            # New Relic NerdGraph API
    │   ├── claude/              # Claude API 分析器
    │   ├── openai/              # OpenAI API 分析器
    │   ├── local/
    │   │   ├── analyzer.rs      # LocalAnalyzer（规则组合器）
    │   │   └── rules/
    │   │       ├── rule.rs      # trait LocalRule
    │   │       ├── sensitive.rs # 敏感数据检测
    │   │       └── endpoint.rs  # 接口统计
    │   ├── serilog/             # Serilog 本地日志解析
    │   ├── output/              # TerminalWriter, MarkdownWriter
    │   └── shared/
    │       └── prompt.rs        # 共享 Prompt 构建（zh/en）
    └── interfaces/
        └── cli/                 # clap CLI 命令定义
```

---

## 扩展指南

### 添加新的本地分析规则

1. 在 `src/infrastructure/local/rules/` 创建新文件
2. 实现 `LocalRule` trait：

```rust
pub struct MyRule;

impl LocalRule for MyRule {
    fn name(&self) -> &str { "my_rule" }

    fn check(&self, input: &AnalysisInput) -> Vec<Issue> {
        // 分析 input.logs 和 input.errors，返回问题列表
        vec![]
    }

    fn suggestions(&self, input: &AnalysisInput) -> Vec<Suggestion> {
        vec![]
    }
}
```

3. 在 `LocalAnalyzer::with_default_rules()` 中注册

### 添加新的数据源

1. 在 `src/infrastructure/` 创建新模块
2. 实现 `DataSource` trait：

```rust
#[async_trait]
impl DataSource for MyDataSource {
    async fn fetch_logs(&self, range: &QueryRange) -> anyhow::Result<Vec<LogEntry>> { ... }
    async fn fetch_errors(&self, range: &QueryRange) -> anyhow::Result<Vec<ErrorEvent>> { ... }
}
```

3. 在 `main.rs` 中添加对应的 CLI 命令和构建逻辑

### 添加新的分析引擎

1. 在 `src/infrastructure/` 创建新模块
2. 实现 `Analyzer` trait：

```rust
#[async_trait]
impl Analyzer for MyAnalyzer {
    async fn analyze(&self, input: &AnalysisInput) -> anyhow::Result<AnalysisOutput> { ... }
}
```

3. 在 `config.example.toml` 的 `[analyzer]` 中添加新的 `provider` 值
4. 在 `main.rs` 的 `build_analyzer()` 中添加对应分支

### 添加新的输出格式

1. 在 `src/infrastructure/output/` 创建新文件
2. 实现 `ReportWriter` trait：

```rust
#[async_trait]
impl ReportWriter for SlackWriter {
    async fn write(&self, report: &ReportDto) -> anyhow::Result<()> { ... }
}
```

---

## 支持的日志格式

### Serilog JSON (CLEF)

```json
{"@t":"2026-03-27T09:11:49.123Z","@l":"Warning","@m":"Message","SourceContext":"MyApp"}
```

### Serilog 管道分隔格式

```
09:11:49 || Warning || SourceContext || Message content || ExceptionDetails ||end
```

文件命名应包含日期（`log_20260327.log` 或 `log_2026-03-27.log`），用于解析时间戳的日期部分。

---

## 文档

| 文档 | 说明 |
|------|------|
| [clean-architecture.md](docs/clean-architecture.md) | 架构分层规则与扩展点说明 |
| [rust-guidelines.md](docs/rust-guidelines.md) | Rust 编码规范与异步模式 |
| [security.md](docs/security.md) | API Key 安全处理规则 |
| [log-formats.md](docs/log-formats.md) | 支持的日志格式规范 |
