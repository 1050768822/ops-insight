use crate::domain::entities::{ErrorEvent, LogLevel};
use crate::domain::ports::AnalysisInput;
use crate::infrastructure::shared::redaction::redact_for_display;

/// 根据语言配置构建分析 prompt。
/// language: "zh"（默认）或 "en"
pub fn build_prompt(input: &AnalysisInput, language: &str) -> String {
    let total = input.logs.len();
    let count = |level: &LogLevel| input.logs.iter().filter(|e| &e.level == level).count();
    let warn_count = count(&LogLevel::Warn);
    let error_count = count(&LogLevel::Error);
    let fatal_count = count(&LogLevel::Fatal);

    let event_lines: String = input
        .errors
        .iter()
        .take(50)
        .map(|e| event_line(e))
        .collect::<Vec<_>>()
        .join("\n");

    match language {
        "en" => build_en(
            input,
            total,
            warn_count,
            error_count,
            fatal_count,
            &event_lines,
        ),
        _ => build_zh(
            input,
            total,
            warn_count,
            error_count,
            fatal_count,
            &event_lines,
        ),
    }
}

fn build_zh(
    input: &AnalysisInput,
    total: usize,
    warn: usize,
    error: usize,
    fatal: usize,
    events: &str,
) -> String {
    let log_summary =
        format!("共 {total} 条日志 — Warning: {warn}, Error: {error}, Fatal: {fatal}");
    let events_section = if events.is_empty() {
        "无异常事件".to_string()
    } else {
        events.to_string()
    };

    format!(
        r#"你是一位资深 DevOps 工程师。请分析以下服务器日志数据，给出问题列表和优化建议。

## 时间范围
{period}

## 日志统计
{log_summary}

## 异常事件（Warning/Error/Fatal，按频率排序，最多50条）
{events_section}

## 要求
请以 JSON 格式返回分析结果，结构如下：
{{
  "summary": "总体情况的简短描述（2-3句话）",
  "issues": [
    {{
      "severity": "critical|high|medium|low",
      "title": "问题标题",
      "description": "详细描述",
      "affected_hosts": ["host1"],
      "occurrence_count": 数字
    }}
  ],
  "suggestions": [
    {{
      "priority": "High|Medium|Low",
      "title": "建议标题",
      "detail": "具体操作建议"
    }}
  ]
}}

只返回 JSON，不要其他文字。"#,
        period = input.period_label,
        log_summary = log_summary,
        events_section = events_section,
    )
}

fn build_en(
    input: &AnalysisInput,
    total: usize,
    warn: usize,
    error: usize,
    fatal: usize,
    events: &str,
) -> String {
    let log_summary =
        format!("Total {total} log entries — Warning: {warn}, Error: {error}, Fatal: {fatal}");
    let events_section = if events.is_empty() {
        "No significant events".to_string()
    } else {
        events.to_string()
    };

    format!(
        r#"You are a senior DevOps engineer. Analyze the following server log data and provide a list of issues and optimization suggestions.

## Time Range
{period}

## Log Statistics
{log_summary}

## Significant Events (Warning/Error/Fatal, sorted by frequency, up to 50)
{events_section}

## Requirements
Return the analysis as JSON with the following structure:
{{
  "summary": "Brief overall description (2-3 sentences)",
  "issues": [
    {{
      "severity": "critical|high|medium|low",
      "title": "Issue title",
      "description": "Detailed description",
      "affected_hosts": ["host1"],
      "occurrence_count": number
    }}
  ],
  "suggestions": [
    {{
      "priority": "High|Medium|Low",
      "title": "Suggestion title",
      "detail": "Specific action steps"
    }}
  ]
}}

Return only JSON, no other text."#,
        period = input.period_label,
        log_summary = log_summary,
        events_section = events_section,
    )
}

fn event_line(e: &ErrorEvent) -> String {
    let freq = if e.count >= 100 {
        "high-freq"
    } else if e.count >= 10 {
        "mid-freq"
    } else {
        "low-freq"
    };
    let message = redact_for_display(&e.message, 200);
    format!(
        "- [{}x][{}] {} | host: {} | first: {} last: {}",
        e.count,
        freq,
        message,
        e.hostname,
        e.first_seen.format("%m-%d %H:%M"),
        e.last_seen.format("%m-%d %H:%M"),
    )
}
