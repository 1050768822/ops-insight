mod application;
mod domain;
mod infrastructure;
mod interfaces;

use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};
use clap::Parser;
use serde::Deserialize;

use application::dtos::QueryRange;
use application::use_cases::generate_report::{daily_range, weekly_range, GenerateReportUseCase};
use domain::ports::Analyzer;
use domain::value_objects::SecretKey;
use infrastructure::claude::ClaudeAnalyzer;
use infrastructure::newrelic::NewRelicSource;
use infrastructure::openai::OpenAiAnalyzer;
use infrastructure::output::{MarkdownWriter, TerminalWriter};
use infrastructure::serilog::SerilogFileSource;
use interfaces::cli::{Cli, Command, ConfigAction};

// ── Config structs ────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct Config {
    newrelic: NewRelicConfig,
    analyzer: AnalyzerConfig,
    claude: ClaudeConfig,
    openai: OpenAiConfig,
    output: OutputConfig,
    #[serde(default)]
    servers: Vec<ServerConfig>,
}

#[derive(Deserialize, Default, zeroize::ZeroizeOnDrop)]
struct NewRelicConfig {
    api_key: String,
    account_id: String,
}

#[derive(Deserialize, Default)]
struct AnalyzerConfig {
    #[serde(default = "default_provider")]
    provider: String,
}

fn default_provider() -> String {
    "claude".to_string()
}

#[derive(Deserialize, Default, zeroize::ZeroizeOnDrop)]
struct ClaudeConfig {
    api_key: String,
    #[serde(default = "default_claude_model")]
    model: String,
}

fn default_claude_model() -> String {
    "claude-opus-4-6".to_string()
}

#[derive(Deserialize, Default, zeroize::ZeroizeOnDrop)]
struct OpenAiConfig {
    api_key: String,
    #[serde(default = "default_openai_model")]
    model: String,
}

fn default_openai_model() -> String {
    "gpt-4o".to_string()
}

#[derive(Deserialize, Default)]
struct OutputConfig {
    #[serde(default = "default_format")]
    format: String,
    #[serde(default = "default_output_dir")]
    output_dir: String,
    #[serde(default = "default_language")]
    language: String,
}

fn default_format() -> String {
    "markdown".to_string()
}

fn default_output_dir() -> String {
    "./reports".to_string()
}

fn default_language() -> String {
    "zh".to_string()
}

#[derive(Deserialize)]
struct ServerConfig {
    #[allow(dead_code)]
    name: String,
    hostname: String,
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Command::Config { action } => match action {
            ConfigAction::Init => {
                init_config()?;
                return Ok(());
            }
        },
        _ => {}
    }

    let config = load_config(&cli.config)?;
    let analyzer = build_analyzer(&config)?;

    let mut writers: Vec<Arc<dyn domain::ports::ReportWriter>> = vec![Arc::new(TerminalWriter)];
    if config.output.format == "markdown" || config.output.format == "both" {
        writers.push(Arc::new(MarkdownWriter::new(PathBuf::from(
            &config.output.output_dir,
        ))));
    }

    match &cli.command {
        Command::Serilog { path, from, to } => {
            let source = Arc::new(SerilogFileSource::new(PathBuf::from(path)));
            let range = serilog_range(from.as_deref(), to.as_deref())?;
            GenerateReportUseCase::new(source, analyzer, writers)
                .execute(range)
                .await?;
        }
        _ => {
            if config.servers.is_empty() {
                anyhow::bail!(
                    "config.toml 中 [[servers]] 列表不能为空，请至少配置一台服务器。"
                );
            }
            let hostnames: Vec<String> =
                config.servers.iter().map(|s| s.hostname.clone()).collect();
            let data_source = Arc::new(NewRelicSource::new(
                SecretKey::new("newrelic_api_key", config.newrelic.api_key.clone()),
                config.newrelic.account_id.clone(),
            ));
            let range = match &cli.command {
                Command::Daily => daily_range(hostnames),
                Command::Weekly => weekly_range(hostnames),
                Command::Custom { from, to } => parse_custom_range(from, to, hostnames)?,
                Command::Config { .. } | Command::Serilog { .. } => unreachable!(),
            };
            GenerateReportUseCase::new(data_source, analyzer, writers)
                .execute(range)
                .await?;
        }
    }

    Ok(())
}

// ── Analyzer factory ──────────────────────────────────────────────────────────

fn build_analyzer(config: &Config) -> anyhow::Result<Arc<dyn Analyzer>> {
    let language = config.output.language.clone();
    match config.analyzer.provider.as_str() {
        "claude" => {
            if config.claude.api_key.is_empty() {
                anyhow::bail!("analyzer.provider = \"claude\" 但 [claude] api_key 未配置");
            }
            Ok(Arc::new(ClaudeAnalyzer::new(
                SecretKey::new("claude_api_key", config.claude.api_key.clone()),
                config.claude.model.clone(),
                language,
            )))
        }
        "openai" => {
            if config.openai.api_key.is_empty() {
                anyhow::bail!("analyzer.provider = \"openai\" 但 [openai] api_key 未配置");
            }
            Ok(Arc::new(OpenAiAnalyzer::new(
                SecretKey::new("openai_api_key", config.openai.api_key.clone()),
                config.openai.model.clone(),
                language,
            )))
        }
        other => anyhow::bail!(
            "未知的 analyzer.provider = \"{other}\"，支持: \"claude\" | \"openai\""
        ),
    }
}

// ── Range helpers ─────────────────────────────────────────────────────────────

fn parse_custom_range(from: &str, to: &str, hostnames: Vec<String>) -> anyhow::Result<QueryRange> {
    let from_dt = NaiveDate::parse_from_str(from, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("--from 格式错误，请使用 YYYY-MM-DD"))?
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let to_dt = NaiveDate::parse_from_str(to, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("--to 格式错误，请使用 YYYY-MM-DD"))?
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc();
    Ok(QueryRange { from: from_dt, to: to_dt, hostnames })
}

fn serilog_range(from: Option<&str>, to: Option<&str>) -> anyhow::Result<QueryRange> {
    let from_dt = match from {
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("--from 格式错误，请使用 YYYY-MM-DD"))?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
        None => DateTime::from_timestamp(0, 0).unwrap(),
    };
    let to_dt = match to {
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("--to 格式错误，请使用 YYYY-MM-DD"))?
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc(),
        None => Utc::now(),
    };
    Ok(QueryRange { from: from_dt, to: to_dt, hostnames: vec![] })
}

// ── Config loading ────────────────────────────────────────────────────────────

fn load_config(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path).map_err(|_| {
        anyhow::anyhow!("找不到配置文件 '{path}'，请先运行: ops-insight config init")
    })?;
    let mut config: Config = toml::from_str(&content)?;

    // 优先级：Keychain > 环境变量 > 配置文件
    if let Some(v) = keychain_get("newrelic_api_key") {
        config.newrelic.api_key = v;
    } else if let Ok(v) = std::env::var("NEWRELIC_API_KEY") {
        config.newrelic.api_key = v;
    }
    if let Ok(v) = std::env::var("NEWRELIC_ACCOUNT_ID") {
        config.newrelic.account_id = v;
    }
    if let Some(v) = keychain_get("claude_api_key") {
        config.claude.api_key = v;
    } else if let Ok(v) = std::env::var("CLAUDE_API_KEY") {
        config.claude.api_key = v;
    }
    if let Some(v) = keychain_get("openai_api_key") {
        config.openai.api_key = v;
    } else if let Ok(v) = std::env::var("OPENAI_API_KEY") {
        config.openai.api_key = v;
    }

    Ok(config)
}

fn keychain_get(service: &str) -> Option<String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-a", &whoami(), "-s", service, "-w"])
        .output()
        .ok()?;
    if output.status.success() {
        let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

fn whoami() -> String {
    std::env::var("USER").unwrap_or_else(|_| "ops-insight".to_string())
}

fn init_config() -> anyhow::Result<()> {
    let template = include_str!("../config.example.toml");
    if std::path::Path::new("config.toml").exists() {
        eprintln!("config.toml 已存在，跳过。");
    } else {
        std::fs::write("config.toml", template)?;
        println!("已生成 config.toml，请填写 API Key 后运行报告命令。");
    }
    Ok(())
}
