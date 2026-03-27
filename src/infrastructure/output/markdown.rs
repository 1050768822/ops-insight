use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::application::dtos::ReportDto;
use crate::domain::entities::Severity;
use crate::domain::ports::ReportWriter;

pub struct MarkdownWriter {
    output_dir: PathBuf,
}

impl MarkdownWriter {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    fn render(report: &ReportDto) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {}\n\n", report.title));
        md.push_str(&format!(
            "> 时间范围：{} ~ {}\n\n",
            report.period.from.format("%Y-%m-%d %H:%M UTC"),
            report.period.to.format("%Y-%m-%d %H:%M UTC"),
        ));
        md.push_str(&format!("{}\n\n", report.summary));

        if !report.issues.is_empty() {
            md.push_str(&format!("## 发现问题 ({})\n\n", report.issues.len()));
            md.push_str("| 严重度 | 问题 | 次数 | 影响主机 |\n");
            md.push_str("|--------|------|------|----------|\n");
            for issue in &report.issues {
                md.push_str(&format!(
                    "| {} | **{}** | {} | {} |\n",
                    severity_label(&issue.severity),
                    issue.title,
                    issue.occurrence_count,
                    issue.affected_hosts.join(", "),
                ));
            }
            md.push('\n');

            for issue in &report.issues {
                md.push_str(&format!("### {}\n\n", issue.title));
                md.push_str(&format!("{}\n\n", issue.description));
            }
        }

        if !report.suggestions.is_empty() {
            md.push_str(&format!("## 优化建议 ({})\n\n", report.suggestions.len()));
            for (i, s) in report.suggestions.iter().enumerate() {
                md.push_str(&format!(
                    "{}. **[{}]** {}\n\n   {}\n\n",
                    i + 1,
                    s.priority,
                    s.title,
                    s.detail,
                ));
            }
        }

        md
    }
}

#[async_trait]
impl ReportWriter for MarkdownWriter {
    async fn write(&self, report: &ReportDto) -> anyhow::Result<()> {
        fs::create_dir_all(&self.output_dir).await?;

        let filename = format!("report_{}.md", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let path = self.output_dir.join(&filename);

        fs::write(&path, Self::render(report)).await?;

        println!("报告已保存：{}", path.display());
        Ok(())
    }
}

fn severity_label(s: &Severity) -> &'static str {
    match s {
        Severity::Critical => "🔴 危急",
        Severity::High => "🟠 高",
        Severity::Medium => "🟡 中",
        Severity::Low => "🟢 低",
    }
}
