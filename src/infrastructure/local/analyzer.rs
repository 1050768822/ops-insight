use std::sync::Arc;

use async_trait::async_trait;

use super::rules::LocalRule;
use crate::domain::ports::{AnalysisInput, AnalysisOutput, Analyzer};

pub struct LocalAnalyzer {
    rules: Vec<Arc<dyn LocalRule>>,
    language: String,
}

impl LocalAnalyzer {
    pub fn new(rules: Vec<Arc<dyn LocalRule>>, language: String) -> Self {
        Self { rules, language }
    }

    /// 使用默认规则集构建（敏感数据 + 接口统计）
    pub fn with_default_rules(language: String) -> Self {
        use super::rules::{EndpointStatsRule, SensitiveDataRule};
        Self::new(
            vec![
                Arc::new(SensitiveDataRule),
                Arc::new(EndpointStatsRule::default()),
            ],
            language,
        )
    }
}

#[async_trait]
impl Analyzer for LocalAnalyzer {
    async fn analyze(&self, input: &AnalysisInput) -> anyhow::Result<AnalysisOutput> {
        let mut all_issues = Vec::new();
        let mut all_suggestions = Vec::new();

        for rule in &self.rules {
            tracing::info!(rule = rule.name(), "运行本地分析规则");
            let mut issues = rule.check(input);
            let mut suggestions = rule.suggestions(input);
            all_issues.append(&mut issues);
            all_suggestions.append(&mut suggestions);
        }

        // 按严重度降序排列
        all_issues.sort_by(|a, b| {
            b.severity
                .partial_cmp(&a.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let summary = build_summary(&all_issues, input, &self.language);

        Ok(AnalysisOutput {
            summary,
            issues: all_issues,
            suggestions: all_suggestions,
        })
    }
}

fn build_summary(
    issues: &[crate::domain::entities::Issue],
    input: &AnalysisInput,
    language: &str,
) -> String {
    use crate::domain::entities::Severity;

    let critical = issues
        .iter()
        .filter(|i| i.severity == Severity::Critical)
        .count();
    let high = issues
        .iter()
        .filter(|i| i.severity == Severity::High)
        .count();
    let medium = issues
        .iter()
        .filter(|i| i.severity == Severity::Medium)
        .count();

    match language {
        "en" => format!(
            "Local analysis of {} log entries (period: {}). Found {} issues: {} critical, {} high, {} medium.",
            input.logs.len(),
            input.period_label,
            issues.len(),
            critical,
            high,
            medium,
        ),
        _ => format!(
            "本地分析完成，共扫描 {} 条日志（{}）。发现 {} 个问题：危急 {}，高 {}，中 {}。",
            input.logs.len(),
            input.period_label,
            issues.len(),
            critical,
            high,
            medium,
        ),
    }
}
