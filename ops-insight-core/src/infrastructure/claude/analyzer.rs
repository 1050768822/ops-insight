use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::PromptConfig;
use crate::domain::entities::{Issue, Priority, Severity, Suggestion};
use crate::domain::ports::{AnalysisInput, AnalysisOutput, Analyzer};
use crate::domain::value_objects::SecretKey;
use crate::infrastructure::shared::redaction::redact_for_display;

pub struct ClaudeAnalyzer {
    api_key: SecretKey,
    model: String,
    language: String,
    prompt_config: PromptConfig,
    client: reqwest::Client,
}

impl ClaudeAnalyzer {
    pub fn new(api_key: SecretKey, model: String, language: String, prompt_config: PromptConfig) -> Self {
        Self {
            api_key,
            model,
            language,
            prompt_config,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

#[derive(Deserialize)]
struct RawAnalysis {
    summary: String,
    issues: Vec<RawIssue>,
    suggestions: Vec<RawSuggestion>,
}

#[derive(Deserialize)]
struct RawIssue {
    severity: String,
    title: String,
    description: String,
    affected_hosts: Vec<String>,
    occurrence_count: u64,
}

#[derive(Deserialize)]
struct RawSuggestion {
    priority: String,
    title: String,
    detail: String,
}

#[async_trait]
impl Analyzer for ClaudeAnalyzer {
    async fn analyze(&self, input: &AnalysisInput) -> anyhow::Result<AnalysisOutput> {
        let prompt = crate::infrastructure::shared::prompt::build_prompt(
            input,
            &self.language,
            &self.prompt_config,
        );

        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 2048,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let http_resp = self
            .api_key
            .use_key("claude_analyze_request", |key| {
                self.client
                    .post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&request)
                    .send()
            })
            .await?;

        let status = http_resp.status();
        let body = http_resp.text().await?;
        let safe_body = redact_for_display(&body, 300);

        if !status.is_success() {
            anyhow::bail!("Claude API 错误 {status}，响应摘要: {safe_body}");
        }

        let resp: ClaudeResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Claude 响应解析失败: {e}；响应摘要: {safe_body}"))?;

        let text = resp
            .content
            .into_iter()
            .next()
            .map(|c| c.text)
            .unwrap_or_default();

        let raw: RawAnalysis = serde_json::from_str(&text).map_err(|e| {
            let safe_text = redact_for_display(&text, 300);
            anyhow::anyhow!("Claude 返回格式解析失败: {e}；内容摘要: {safe_text}")
        })?;

        let issues = raw
            .issues
            .into_iter()
            .map(|r| Issue {
                severity: parse_severity(&r.severity),
                title: r.title,
                description: r.description,
                affected_hosts: r.affected_hosts,
                occurrence_count: r.occurrence_count,
            })
            .collect();

        let suggestions = raw
            .suggestions
            .into_iter()
            .map(|r| Suggestion {
                priority: parse_priority(&r.priority),
                title: r.title,
                detail: r.detail,
            })
            .collect();

        Ok(AnalysisOutput {
            summary: raw.summary,
            issues,
            suggestions,
        })
    }
}

fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "medium" => Severity::Medium,
        _ => Severity::Low,
    }
}

fn parse_priority(s: &str) -> Priority {
    match s.to_lowercase().as_str() {
        "high" => Priority::High,
        "medium" => Priority::Medium,
        _ => Priority::Low,
    }
}
