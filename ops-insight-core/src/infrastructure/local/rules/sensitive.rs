use super::rule::LocalRule;
use crate::config::DesensitizeConfig;
use crate::domain::entities::{Issue, Priority, Severity, Suggestion};
use crate::domain::ports::AnalysisInput;
use crate::infrastructure::shared::redaction::redact_for_display;
use regex::Regex;

/// 所有内置敏感数据检测模式 (pattern, label)
const BUILTIN_PATTERNS: &[(&str, &str)] = &[
    (r"(?i)password\s*[=:]\s*\S+", "密码明文"),
    (r"(?i)passwd\s*[=:]\s*\S+", "密码明文"),
    (r"(?i)pwd\s*[=:]\s*\S+", "密码明文"),
    (r"(?i)secret\s*[=:]\s*\S+", "Secret Key"),
    (r"(?i)api[_-]?key\s*[=:]\s*\S+", "API Key"),
    (r"sk-[A-Za-z0-9]{20,}", "OpenAI API Key"),
    (r"NRAK-[A-Za-z0-9]{20,}", "New Relic API Key"),
    (r"sk-ant-[A-Za-z0-9\-]{20,}", "Anthropic API Key"),
    (
        r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}",
        "邮箱地址",
    ),
    (r"(?i)bearer\s+[A-Za-z0-9\-._~+/]+=*", "Bearer Token"),
    (
        r"eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+",
        "JWT Token",
    ),
    (r"(?i)connectionstring\s*[=:]\s*\S+", "数据库连接串"),
    (r"(?i)Server=.{5,};.{0,50}Password=", "数据库连接串"),
    (
        r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13})\b",
        "信用卡号",
    ),
];

pub struct SensitiveDataRule {
    cfg: DesensitizeConfig,
}

impl SensitiveDataRule {
    pub fn new(cfg: DesensitizeConfig) -> Self {
        Self { cfg }
    }
}

impl Default for SensitiveDataRule {
    fn default() -> Self {
        Self {
            cfg: DesensitizeConfig::default(),
        }
    }
}

impl LocalRule for SensitiveDataRule {
    fn name(&self) -> &str {
        "sensitive_data"
    }

    fn check(&self, input: &AnalysisInput) -> Vec<Issue> {
        if !self.cfg.enabled {
            return vec![];
        }

        // 过滤掉被禁用的内置规则（同 label 可有多个 regex，findings 会合并）
        let patterns: Vec<(Regex, &str)> = BUILTIN_PATTERNS
            .iter()
            .filter(|(_, label)| !self.cfg.disabled_builtin.iter().any(|d| d == *label))
            .filter_map(|(pat, label)| Regex::new(pat).ok().map(|r| (r, *label)))
            .collect();

        // 追加用户自定义 pattern
        let custom_compiled: Vec<(Regex, String)> = self
            .cfg
            .custom_patterns
            .iter()
            .filter(|p| p.enabled)
            .filter_map(|p| Regex::new(&p.pattern).ok().map(|r| (r, p.name.clone())))
            .collect();

        struct LabelFindings {
            count: u64,
            sample: String,
            hosts: Vec<String>,
        }

        let mut findings: std::collections::HashMap<String, LabelFindings> =
            std::collections::HashMap::new();

        for entry in &input.logs {
            for (regex, label) in &patterns {
                if regex.is_match(&entry.message) {
                    let record = findings
                        .entry(label.to_string())
                        .or_insert_with(|| LabelFindings {
                            count: 0,
                            sample: entry.message.clone(),
                            hosts: Vec::new(),
                        });
                    record.count += 1;
                    if !record.hosts.contains(&entry.hostname) {
                        record.hosts.push(entry.hostname.clone());
                    }
                }
            }
            for (regex, label) in &custom_compiled {
                if regex.is_match(&entry.message) {
                    let record = findings
                        .entry(label.clone())
                        .or_insert_with(|| LabelFindings {
                            count: 0,
                            sample: entry.message.clone(),
                            hosts: Vec::new(),
                        });
                    record.count += 1;
                    if !record.hosts.contains(&entry.hostname) {
                        record.hosts.push(entry.hostname.clone());
                    }
                }
            }
        }

        findings
            .into_iter()
            .map(|(label, data)| {
                let severity = match label.as_str() {
                    "邮箱地址" | "信用卡号" => Severity::Medium,
                    _ => Severity::High,
                };

                let sample = redact_for_display(&data.sample, 100);
                let description = format!("发现 {} 处匹配，脱敏示例：{}", data.count, sample);

                Issue {
                    severity,
                    title: format!("日志中发现 {} 泄漏", label),
                    description,
                    affected_hosts: data.hosts,
                    occurrence_count: data.count,
                }
            })
            .collect()
    }

    fn suggestions(&self, input: &AnalysisInput) -> Vec<Suggestion> {
        if self.check(input).is_empty() {
            return vec![];
        }

        vec![
            Suggestion {
                priority: Priority::High,
                title: "立即轮换泄漏的凭证".to_string(),
                detail: "检查日志中出现的所有凭证，立即在相关系统中完成轮换，并审查日志采集管道是否存在脱敏缺失的环节。".to_string(),
            },
            Suggestion {
                priority: Priority::High,
                title: "配置日志脱敏中间件".to_string(),
                detail: "在日志写出前引入脱敏中间件，对密码、密钥、Token 等敏感字段进行掩码处理，防止明文信息进入日志存储。".to_string(),
            },
        ]
    }
}

/// 返回所有内置规则的标签列表（供前端展示）
pub fn builtin_pattern_labels() -> Vec<&'static str> {
    let mut labels: Vec<&str> = Vec::new();
    for (_, label) in BUILTIN_PATTERNS {
        if !labels.contains(label) {
            labels.push(label);
        }
    }
    labels
}
