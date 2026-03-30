use std::sync::Arc;

use crate::config::Config;
use crate::domain::ports::Analyzer;
use crate::domain::value_objects::SecretKey;
use crate::infrastructure::claude::ClaudeAnalyzer;
use crate::infrastructure::local::LocalAnalyzer;
use crate::infrastructure::openai::OpenAiAnalyzer;

pub fn build_analyzer(config: &Config) -> anyhow::Result<Arc<dyn Analyzer>> {
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
        "local" => Ok(Arc::new(LocalAnalyzer::with_default_rules(language))),
        other => anyhow::bail!(
            "未知的 analyzer.provider = \"{other}\"，支持: \"claude\" | \"openai\" | \"local\""
        ),
    }
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
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

pub fn keychain_get(service: &str) -> Option<String> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-a",
            &whoami(),
            "-s",
            service,
            "-w",
        ])
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

pub fn whoami() -> String {
    std::env::var("USER").unwrap_or_else(|_| "ops-insight".to_string())
}
