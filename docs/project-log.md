# ops-insight 项目日志

---

## 2026-03-30 — 添加 Tauri 2.0 桌面 GUI

### 背景

原项目为纯 CLI 工具，通过终端命令生成运维报告。为使报告生成、日志分析和配置管理可视化操作，本次为项目引入 Tauri 2.0 桌面 GUI，同时保持 CLI 完整可用。

---

### 结构变更

**转换为 Cargo Workspace**

原单包结构改为三成员工作空间：

```
ops-insight/
├── Cargo.toml                  # workspace manifest（新）
├── ops-insight-core/           # 共享库 + CLI 二进制（原 src/ 迁移而来）
├── src-tauri/                  # Tauri 2.0 桌面应用（新）
└── frontend/                   # Web UI，纯 HTML/CSS/JS（新）
```

原 `src/` 目录在迁移完成后删除，内容全部移入 `ops-insight-core/src/`。

---

### ops-insight-core 重构详情

**新增模块（从 `main.rs` 提取）**

| 文件 | 内容 |
|------|------|
| `src/config.rs` | `Config` / `NewRelicConfig` / `ClaudeConfig` / `OpenAiConfig` / `OutputConfig` / `AnalyzerConfig` / `ServerConfig`，字段改为 `pub` |
| `src/factory.rs` | `build_analyzer()` / `load_config()` / `keychain_get()` / `whoami()`，改为 `pub` |
| `src/helpers.rs` | `parse_custom_range()` / `serilog_range()` / `init_config()`，改为 `pub` |
| `src/lib.rs` | 公开上述所有模块及基础设施层类型，作为 Tauri crate 的依赖入口 |

**`src/main.rs` 精简**

移除所有内联定义，改为通过 `ops_insight_core::` 导入，仅保留 `#[tokio::main]` 入口和 CLI 路由逻辑。

**`application/dtos/mod.rs` 变更**

`ReportDto` / `IssueDto` / `SuggestionDto` 新增 `Clone` derive，以支持 Tauri 命令缓存报告至 `AppState`。

---

### src-tauri 详情

**依赖**
- `tauri = "2"`
- `tauri-plugin-dialog = "2"` — 原生文件选择对话框
- `tauri-plugin-fs = "2"` — 配置文件读写
- `tauri-plugin-shell = "2"` — 可选：在访达中打开报告目录
- `ops-insight-core = { path = "../ops-insight-core" }` — 共享业务逻辑

**Tauri 命令**

报告生成（4 个 async 命令，返回 `ReportDto`）：

| 命令 | 说明 |
|------|------|
| `generate_daily_report` | 昨日报告，使用 NewRelic 数据源 |
| `generate_weekly_report` | 过去 7 天报告，使用 NewRelic 数据源 |
| `generate_custom_report(from, to)` | 自定义日期范围，使用 NewRelic 数据源 |
| `generate_serilog_report(path, from?, to?)` | 本地 Serilog 文件分析 |

配置管理（4 个同步命令）：

| 命令 | 说明 |
|------|------|
| `load_config_cmd` | 读取原始 TOML 文本（不解析 Keychain，避免密钥泄露至前端） |
| `save_config_cmd(content)` | 验证 TOML 格式后写入磁盘 |
| `init_config_cmd` | 从 `config.example.toml` 生成初始配置 |
| `get_config_path` | 返回配置文件路径（供界面展示） |

**AppState**

```rust
pub struct AppState {
    pub config_path: PathBuf,          // 默认: app_config_dir()/config.toml
    pub last_report: Mutex<Option<ReportDto>>,  // 最近一次报告缓存
}
```

**安全考量**

- GUI 中报告命令不添加 `TerminalWriter`，避免向 GUI bundle 的 stdout 输出
- `load_config_cmd` 直接读 TOML 文件原文，不调用 `load_config()`（后者会解析 macOS Keychain 并将密钥写入内存），防止密钥通过 IPC 泄漏至 JavaScript 层

---

### frontend 详情

三个功能 Tab：

**生成报告**
- 单选报告类型：昨日 / 周报（7天） / 自定义范围 / Serilog 文件
- 自定义范围时展示日期选择器
- Serilog 模式时展示原生文件选择按钮及可选日期过滤
- 生成中显示 spinner，错误显示红色 banner

**报告详情**
- 标题 / 时间范围 / 摘要
- Issue 卡片，severity 色标：
  - 🔴 危急（`#dc2626`）
  - 🟠 高（`#ea580c`）
  - 🟡 中（`#d97706`）
  - 🟢 低（`#16a34a`）
- 优化建议列表，按 priority 左边框颜色区分

**配置管理**
- 展示配置文件路径
- TOML 编辑器（monospace textarea）
- 保存 / 生成模板 / 重新加载 三个操作按钮

---

### 构建命令

```bash
# 安装 tauri-cli（首次）
cargo install tauri-cli --version '^2'

# 开发模式启动 GUI
cargo tauri dev

# 生产构建（生成 .app bundle）
cargo tauri build

# 仅构建 CLI（同之前）
cargo build -p ops-insight-core --bin ops-insight
```

---

## 2026-03-30 — feat: add llm body handle

（见 commit b31efec）

## 2026-03-30 — feat: add ops init

（见 commit 4ca438f）
