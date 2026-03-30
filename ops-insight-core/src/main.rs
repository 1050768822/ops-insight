use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;

use ops_insight_core::application::dtos::QueryRange;
use ops_insight_core::application::use_cases::generate_report::{
    GenerateReportUseCase, daily_range, weekly_range,
};
use ops_insight_core::domain::ports::ReportWriter;
use ops_insight_core::domain::value_objects::SecretKey;
use ops_insight_core::factory::{build_analyzer, load_config};
use ops_insight_core::helpers::{init_config, parse_custom_range, serilog_range};
use ops_insight_core::infrastructure::newrelic::NewRelicSource;
use ops_insight_core::infrastructure::output::{MarkdownWriter, TerminalWriter};
use ops_insight_core::infrastructure::serilog::SerilogFileSource;
use ops_insight_core::interfaces::cli::{Cli, Command, ConfigAction};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Command::Config { action } => match action {
            ConfigAction::Init => {
                init_config()?;
                return Ok(());
            }
        },
        _ => {}
    }

    let config = load_config(&cli.config)?;
    let analyzer = build_analyzer(&config)?;

    let mut writers: Vec<Arc<dyn ReportWriter>> = vec![Arc::new(TerminalWriter)];
    if config.output.format == "markdown" || config.output.format == "both" {
        writers.push(Arc::new(MarkdownWriter::new(PathBuf::from(
            &config.output.output_dir,
        ))));
    }

    match &cli.command {
        Command::Serilog { path, from, to } => {
            let source = Arc::new(SerilogFileSource::new(PathBuf::from(path)));
            let range = serilog_range(from.as_deref(), to.as_deref())?;
            GenerateReportUseCase::new(source, analyzer, writers)
                .execute(range)
                .await?;
        }
        _ => {
            if config.servers.is_empty() {
                anyhow::bail!("config.toml 中 [[servers]] 列表不能为空，请至少配置一台服务器。");
            }
            let hostnames: Vec<String> =
                config.servers.iter().map(|s| s.hostname.clone()).collect();
            let data_source = Arc::new(NewRelicSource::new(
                SecretKey::new("newrelic_api_key", config.newrelic.api_key.clone()),
                config.newrelic.account_id.clone(),
            ));
            let range: QueryRange = match &cli.command {
                Command::Daily => daily_range(hostnames),
                Command::Weekly => weekly_range(hostnames),
                Command::Custom { from, to } => parse_custom_range(from, to, hostnames)?,
                Command::Config { .. } | Command::Serilog { .. } => unreachable!(),
            };
            GenerateReportUseCase::new(data_source, analyzer, writers)
                .execute(range)
                .await?;
        }
    }

    Ok(())
}
