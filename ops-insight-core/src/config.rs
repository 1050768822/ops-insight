use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default)]
pub struct Config {
    pub newrelic: NewRelicConfig,
    pub analyzer: AnalyzerConfig,
    pub claude: ClaudeConfig,
    pub openai: OpenAiConfig,
    pub output: OutputConfig,
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
    #[serde(default)]
    pub desensitize: DesensitizeConfig,
}

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct DesensitizeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 要禁用的内置规则名称列表（如 "密码明文"、"JWT Token"）
    #[serde(default)]
    pub disabled_builtin: Vec<String>,
    /// 用户自定义的正则模式
    #[serde(default)]
    pub custom_patterns: Vec<PatternConfig>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PatternConfig {
    pub name: String,
    pub pattern: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Default, zeroize::ZeroizeOnDrop)]
pub struct NewRelicConfig {
    pub api_key: String,
    pub account_id: String,
}

#[derive(Deserialize, Default)]
pub struct AnalyzerConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
}

fn default_provider() -> String {
    "claude".to_string()
}

#[derive(Deserialize, Default, zeroize::ZeroizeOnDrop)]
pub struct ClaudeConfig {
    pub api_key: String,
    #[serde(default = "default_claude_model")]
    pub model: String,
}

fn default_claude_model() -> String {
    "claude-opus-4-6".to_string()
}

#[derive(Deserialize, Default, zeroize::ZeroizeOnDrop)]
pub struct OpenAiConfig {
    pub api_key: String,
    #[serde(default = "default_openai_model")]
    pub model: String,
}

fn default_openai_model() -> String {
    "gpt-4o".to_string()
}

#[derive(Deserialize, Default)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_language")]
    pub language: String,
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
pub struct ServerConfig {
    #[allow(dead_code)]
    pub name: String,
    pub hostname: String,
}
