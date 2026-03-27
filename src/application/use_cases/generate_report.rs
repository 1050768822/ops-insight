use std::sync::Arc;

use chrono::Utc;

use crate::application::dtos::{IssueDto, QueryRange, ReportDto, SuggestionDto};
use crate::domain::entities::ReportPeriod;
use crate::domain::ports::{AnalysisInput, Analyzer, DataSource, ReportWriter};

pub struct GenerateReportUseCase {
    pub data_source: Arc<dyn DataSource>,
    pub analyzer: Arc<dyn Analyzer>,
    pub writers: Vec<Arc<dyn ReportWriter>>,
}

impl GenerateReportUseCase {
    pub fn new(
        data_source: Arc<dyn DataSource>,
        analyzer: Arc<dyn Analyzer>,
        writers: Vec<Arc<dyn ReportWriter>>,
    ) -> Self {
        Self {
            data_source,
            analyzer,
            writers,
        }
    }

    pub async fn execute(&self, range: QueryRange) -> anyhow::Result<ReportDto> {
        let logs = self.data_source.fetch_logs(&range).await?;
        let errors = self.data_source.fetch_errors(&range).await?;

        let input = AnalysisInput {
            logs,
            errors,
            period_label: range.label(),
        };

        let output = self.analyzer.analyze(&input).await?;

        let report = ReportDto {
            title: format!("运维报告 — {}", range.label()),
            period: ReportPeriod {
                from: range.from,
                to: range.to,
            },
            summary: output.summary,
            issues: output.issues.into_iter().map(IssueDto::from).collect(),
            suggestions: output
                .suggestions
                .into_iter()
                .map(SuggestionDto::from)
                .collect(),
        };

        for writer in &self.writers {
            writer.write(&report).await?;
        }

        Ok(report)
    }
}

pub fn daily_range(hostnames: Vec<String>) -> QueryRange {
    let now = Utc::now();
    let yesterday = now - chrono::Duration::days(1);
    QueryRange {
        from: yesterday
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
        to: yesterday
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc(),
        hostnames,
    }
}

pub fn weekly_range(hostnames: Vec<String>) -> QueryRange {
    let now = Utc::now();
    let week_ago = now - chrono::Duration::days(7);
    QueryRange {
        from: week_ago
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc(),
        to: now
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc(),
        hostnames,
    }
}
