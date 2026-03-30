use async_trait::async_trait;

use crate::application::dtos::ReportDto;

#[async_trait]
pub trait ReportWriter: Send + Sync {
    async fn write(&self, report: &ReportDto) -> anyhow::Result<()>;
}
