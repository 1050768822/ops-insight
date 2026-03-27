use async_trait::async_trait;

use crate::application::dtos::QueryRange;
use crate::domain::entities::{ErrorEvent, LogEntry};

#[async_trait]
pub trait DataSource: Send + Sync {
    async fn fetch_logs(&self, range: &QueryRange) -> anyhow::Result<Vec<LogEntry>>;
    async fn fetch_errors(&self, range: &QueryRange) -> anyhow::Result<Vec<ErrorEvent>>;
}
