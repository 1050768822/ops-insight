use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::entities::{ErrorEvent, Issue, LogEntry, Suggestion};

#[derive(Debug)]
pub struct AnalysisInput {
    pub logs: Vec<LogEntry>,
    pub errors: Vec<ErrorEvent>,
    pub period_label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisOutput {
    pub summary: String,
    pub issues: Vec<Issue>,
    pub suggestions: Vec<Suggestion>,
}

#[async_trait]
pub trait Analyzer: Send + Sync {
    async fn analyze(&self, input: &AnalysisInput) -> anyhow::Result<AnalysisOutput>;
}
