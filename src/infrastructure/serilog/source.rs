use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use tokio::fs;

use crate::application::dtos::QueryRange;
use crate::domain::entities::{ErrorEvent, LogEntry, LogLevel};
use crate::domain::ports::DataSource;

pub struct SerilogFileSource {
    dir: PathBuf,
}

impl SerilogFileSource {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    async fn read_all_entries(&self) -> anyhow::Result<Vec<LogEntry>> {
        let mut entries = Vec::new();

        let metadata = fs::metadata(&self.dir).await.map_err(|e| {
            anyhow::anyhow!("无法访问路径 '{}': {e}", self.dir.display())
        })?;

        if metadata.is_file() {
            // 单文件模式
            match parse_file(&self.dir).await {
                Ok(mut file_entries) => entries.append(&mut file_entries),
                Err(e) => anyhow::bail!("解析文件 '{}' 失败: {e}", self.dir.display()),
            }
        } else {
            // 文件夹模式
            let mut dir = fs::read_dir(&self.dir).await.map_err(|e| {
                anyhow::anyhow!("无法读取目录 '{}': {e}", self.dir.display())
            })?;
            while let Some(entry) = dir.next_entry().await? {
                let path = entry.path();
                if is_log_file(&path) {
                    match parse_file(&path).await {
                        Ok(mut file_entries) => entries.append(&mut file_entries),
                        Err(e) => tracing::warn!(file = %path.display(), "跳过文件: {e}"),
                    }
                }
            }
        }

        entries.sort_by_key(|e| e.timestamp);
        Ok(entries)
    }
}

fn is_log_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("json") | Some("log") | Some("txt")
    )
}

async fn parse_file(path: &Path) -> anyhow::Result<Vec<LogEntry>> {
    let content = fs::read_to_string(path).await?;
    let fallback_hostname = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // 从文件名尝试提取日期，例如 app-20260327.log 或 log_2026-03-27.txt
    let file_date = extract_date_from_filename(path);

    let first_line = content.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let entries = if first_line.trim_start().starts_with('{') {
        parse_json_format(&content, &fallback_hostname)
    } else {
        parse_pipe_format(&content, &fallback_hostname, file_date)
    };

    tracing::info!(file = %path.display(), count = entries.len(), "读取日志文件");
    Ok(entries)
}

// ── JSON 格式 ────────────────────────────────────────────────────────────────

fn parse_json_format(content: &str, fallback_hostname: &str) -> Vec<LogEntry> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| parse_json_line(line, fallback_hostname).ok())
        .collect()
}

fn parse_json_line(line: &str, fallback_hostname: &str) -> anyhow::Result<LogEntry> {
    let v: serde_json::Value = serde_json::from_str(line)?;

    let timestamp = {
        let raw = v["@t"]
            .as_str()
            .or_else(|| v["Timestamp"].as_str())
            .ok_or_else(|| anyhow::anyhow!("缺少时间戳"))?;
        raw.parse::<DateTime<Utc>>()
            .or_else(|_| DateTime::parse_from_rfc3339(raw).map(|d| d.with_timezone(&Utc)))
            .map_err(|e| anyhow::anyhow!("时间戳解析失败: {e}"))?
    };

    let level = {
        let raw = v["@l"]
            .as_str()
            .or_else(|| v["Level"].as_str())
            .unwrap_or("")
            .to_lowercase();
        parse_level_str(&raw)
    };

    let message = v["@m"]
        .as_str()
        .or_else(|| v["@mt"].as_str())
        .or_else(|| v["RenderedMessage"].as_str())
        .or_else(|| v["MessageTemplate"].as_str())
        .unwrap_or("")
        .to_string();

    let hostname = v["Properties"]["MachineName"]
        .as_str()
        .or_else(|| v["Properties"]["HostName"].as_str())
        .unwrap_or(fallback_hostname)
        .to_string();

    let service = v["Properties"]["Application"]
        .as_str()
        .or_else(|| v["Properties"]["SourceContext"].as_str())
        .map(|s| s.to_string());

    Ok(LogEntry { timestamp, level, message, hostname, service })
}

// ── Pipe 格式 ─────────────────────────────────────────────────────────────────
//
// 格式: HH:MM:SS || Level || Source || Message || ExceptionDetails\n...\n ||end
//
// 每条日志以时间戳开头，以 "||end" 结束（可跨多行）

fn parse_pipe_format(
    content: &str,
    fallback_hostname: &str,
    file_date: Option<NaiveDate>,
) -> Vec<LogEntry> {
    // 将全文按 "||end" 分割为独立的日志块
    let blocks: Vec<&str> = content
        .split("||end")
        .map(|b| b.trim())
        .filter(|b| !b.is_empty())
        .collect();

    blocks
        .iter()
        .filter_map(|block| parse_pipe_block(block, fallback_hostname, file_date).ok())
        .collect()
}

fn parse_pipe_block(
    block: &str,
    fallback_hostname: &str,
    file_date: Option<NaiveDate>,
) -> anyhow::Result<LogEntry> {
    // 第一行是主体，后续行是 stack trace
    let first_line = block.lines().next().unwrap_or("").trim();
    let stack_trace: String = block.lines().skip(1).collect::<Vec<_>>().join("\n");

    // 按 " || " 分割第一行
    let parts: Vec<&str> = first_line.splitn(5, " || ").collect();
    if parts.len() < 4 {
        anyhow::bail!("字段不足: {first_line}");
    }

    let time_str = parts[0].trim();
    let level_str = parts[1].trim();
    let source = parts[2].trim();
    let message = parts[3].trim();
    let exception = if parts.len() > 4 {
        let mut ex = parts[4].trim().to_string();
        if !stack_trace.is_empty() {
            ex.push('\n');
            ex.push_str(&stack_trace);
        }
        ex
    } else {
        stack_trace
    };

    let time = NaiveTime::parse_from_str(time_str, "%H:%M:%S")
        .map_err(|e| anyhow::anyhow!("时间解析失败 '{time_str}': {e}"))?;

    let date = file_date.unwrap_or_else(|| Utc::now().date_naive());
    let timestamp = Utc.from_utc_datetime(&date.and_time(time));

    let level = parse_level_str(&level_str.to_lowercase());

    // 将 message + exception 合并，方便后续分析
    let full_message = if exception.is_empty() {
        message.to_string()
    } else {
        // 只保留异常第一行（类型+信息），stack trace 太长不适合聚合
        let ex_summary = exception.lines().next().unwrap_or("").trim();
        format!("{message} || {ex_summary}")
    };

    Ok(LogEntry {
        timestamp,
        level,
        message: full_message,
        hostname: fallback_hostname.to_string(),
        service: Some(source.to_string()),
    })
}

// ── 工具函数 ──────────────────────────────────────────────────────────────────

fn parse_level_str(s: &str) -> LogLevel {
    match s {
        "verbose" | "debug" => LogLevel::Debug,
        "information" | "info" => LogLevel::Info,
        "warning" | "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        "fatal" | "critical" => LogLevel::Fatal,
        _ => LogLevel::Info,
    }
}

/// 从文件名中提取日期，支持常见格式：
/// log-20260327.log / app_2026-03-27.log / service.20260327.txt
fn extract_date_from_filename(path: &Path) -> Option<NaiveDate> {
    let stem = path.file_stem()?.to_str()?;

    // 尝试 YYYY-MM-DD
    let re_dashed = regex::Regex::new(r"(\d{4}-\d{2}-\d{2})").ok()?;
    if let Some(cap) = re_dashed.captures(stem) {
        return NaiveDate::parse_from_str(&cap[1], "%Y-%m-%d").ok();
    }

    // 尝试 YYYYMMDD
    let re_compact = regex::Regex::new(r"(\d{8})").ok()?;
    if let Some(cap) = re_compact.captures(stem) {
        return NaiveDate::parse_from_str(&cap[1], "%Y%m%d").ok();
    }

    None
}

// ── DataSource impl ───────────────────────────────────────────────────────────

#[async_trait]
impl DataSource for SerilogFileSource {
    async fn fetch_logs(&self, range: &QueryRange) -> anyhow::Result<Vec<LogEntry>> {
        let all = self.read_all_entries().await?;
        let filtered = all
            .into_iter()
            .filter(|e| e.timestamp >= range.from && e.timestamp <= range.to)
            .collect();
        Ok(filtered)
    }

    async fn fetch_errors(&self, range: &QueryRange) -> anyhow::Result<Vec<ErrorEvent>> {
        use std::collections::HashMap;

        let logs = self.fetch_logs(range).await?;

        let mut groups: HashMap<(String, String), Vec<LogEntry>> = HashMap::new();
        for entry in logs {
            if matches!(entry.level, LogLevel::Warn | LogLevel::Error | LogLevel::Fatal) {
                groups
                    .entry((entry.message.clone(), entry.hostname.clone()))
                    .or_default()
                    .push(entry);
            }
        }

        let mut errors: Vec<ErrorEvent> = groups
            .into_values()
            .map(|group| {
                let first = group.iter().map(|e| e.timestamp).min().unwrap();
                let last = group.iter().map(|e| e.timestamp).max().unwrap();
                let sample = &group[0];
                ErrorEvent {
                    timestamp: last,
                    message: sample.message.clone(),
                    hostname: sample.hostname.clone(),
                    service: sample.service.clone(),
                    count: group.len() as u64,
                    first_seen: first,
                    last_seen: last,
                }
            })
            .collect();

        // 按出现次数降序排列
        errors.sort_by(|a, b| b.count.cmp(&a.count));
        Ok(errors)
    }
}
