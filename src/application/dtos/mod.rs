use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::entities::{Issue, ReportPeriod, Severity, Suggestion};

#[derive(Debug, Clone)]
pub struct QueryRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub hostnames: Vec<String>,
}

impl QueryRange {
    pub fn label(&self) -> String {
        format!(
            "{} ~ {}",
            self.from.format("%Y-%m-%d"),
            self.to.format("%Y-%m-%d")
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportDto {
    pub title: String,
    pub period: ReportPeriod,
    pub summary: String,
    pub issues: Vec<IssueDto>,
    pub suggestions: Vec<SuggestionDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueDto {
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub affected_hosts: Vec<String>,
    pub occurrence_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestionDto {
    pub priority: String,
    pub title: String,
    pub detail: String,
}

impl From<Issue> for IssueDto {
    fn from(i: Issue) -> Self {
        Self {
            severity: i.severity,
            title: i.title,
            description: i.description,
            affected_hosts: i.affected_hosts,
            occurrence_count: i.occurrence_count,
        }
    }
}

impl From<Suggestion> for SuggestionDto {
    fn from(s: Suggestion) -> Self {
        Self {
            priority: format!("{:?}", s.priority),
            title: s.title,
            detail: s.detail,
        }
    }
}
