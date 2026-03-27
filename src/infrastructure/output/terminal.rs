use async_trait::async_trait;

use crate::application::dtos::ReportDto;
use crate::domain::entities::Severity;
use crate::domain::ports::ReportWriter;

pub struct TerminalWriter;

#[async_trait]
impl ReportWriter for TerminalWriter {
    async fn write(&self, report: &ReportDto) -> anyhow::Result<()> {
        println!("\n{}", "=".repeat(60));
        println!("  {}", report.title);
        println!("{}", "=".repeat(60));
        println!("\n{}\n", report.summary);

        if !report.issues.is_empty() {
            println!("## 发现问题 ({})\n", report.issues.len());
            for issue in &report.issues {
                let badge = severity_badge(&issue.severity);
                println!("[{badge}] {} ({}次)", issue.title, issue.occurrence_count);
                println!("     {}", issue.description);
                if !issue.affected_hosts.is_empty() {
                    println!("     影响主机: {}", issue.affected_hosts.join(", "));
                }
                println!();
            }
        }

        if !report.suggestions.is_empty() {
            println!("## 优化建议 ({})\n", report.suggestions.len());
            for (i, s) in report.suggestions.iter().enumerate() {
                println!("{}. [{}] {}", i + 1, s.priority, s.title);
                println!("   {}", s.detail);
                println!();
            }
        }

        println!("{}", "=".repeat(60));
        Ok(())
    }
}

fn severity_badge(s: &Severity) -> &'static str {
    match s {
        Severity::Critical => "!!危急",
        Severity::High => "! 高  ",
        Severity::Medium => "  中  ",
        Severity::Low => "  低  ",
    }
}
