use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::entities::{Issue, Priority, Severity, Suggestion};
use crate::domain::ports::{AnalysisInput, AnalysisOutput, Analyzer};
use crate::domain::value_objects::SecretKey;
use crate::infrastructure::shared::redaction::redact_for_display;

pub struct OpenAiAnalyzer {
    api_key: SecretKey,
    model: String,
    language: String,
    client: reqwest::Client,
}

impl OpenAiAnalyzer {
    pub fn new(api_key: SecretKey, model: String, language: String) -> Self {
        Self {
            api_key,
            model,
            language,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: u32,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageContent,
}

#[derive(Deserialize)]
struct OpenAiMessageContent {
    content: String,
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
impl Analyzer for OpenAiAnalyzer {
    async fn analyze(&self, input: &AnalysisInput) -> anyhow::Result<AnalysisOutput> {
        let prompt = crate::infrastructure::shared::prompt::build_prompt(input, &self.language);

        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 2048,
        };

        let http_resp = self
            .api_key
            .use_key("openai_analyze_request", |key| {
                self.client
                    .post("https://api.openai.com/v1/chat/completions")
                    .header("Authorization", format!("Bearer {key}"))
                    .header("Content-Type", "application/json")
                    .json(&request)
                    .send()
            })
            .await?;

        let status = http_resp.status();
        let body = http_resp.text().await?;
        let safe_body = redact_for_display(&body, 300);

        if !status.is_success() {
            anyhow::bail!("OpenAI API 错误 {status}，响应摘要: {safe_body}");
        }

        let resp: OpenAiResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("OpenAI 响应解析失败: {e}；响应摘要: {safe_body}"))?;

        let text = resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        let raw: RawAnalysis = serde_json::from_str(&text).map_err(|e| {
            let safe_text = redact_for_display(&text, 300);
            anyhow::anyhow!("OpenAI 返回格式解析失败: {e}；内容摘要: {safe_text}")
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
