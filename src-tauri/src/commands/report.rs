use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;

use ops_insight_core::{
    GenerateReportUseCase, MarkdownWriter, NewRelicSource, ReportDto, SecretKey, SerilogFileSource,
    build_analyzer, daily_range, load_config, parse_custom_range, serilog_range, weekly_range,
};

use crate::state::AppState;

async fn run_report(
    config_path: &PathBuf,
    range: ops_insight_core::QueryRange,
    source: Arc<dyn ops_insight_core::DataSource>,
) -> Result<ReportDto, String> {
    let config = load_config(config_path.to_str().unwrap_or("config.toml"))
        .map_err(|e| e.to_string())?;

    let analyzer = build_analyzer(&config).map_err(|e| e.to_string())?;

    let writers: Vec<Arc<dyn ops_insight_core::ReportWriter>> = vec![Arc::new(
        MarkdownWriter::new(PathBuf::from(&config.output.output_dir)),
    )];

    GenerateReportUseCase::new(source, analyzer, writers)
        .execute(range)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_daily_report(state: State<'_, AppState>) -> Result<ReportDto, String> {
    let config = load_config(state.config_path.to_str().unwrap_or("config.toml"))
        .map_err(|e| e.to_string())?;

    if config.servers.is_empty() {
        return Err("config.toml 中 [[servers]] 列表不能为空".into());
    }
    let hostnames: Vec<String> = config.servers.iter().map(|s| s.hostname.clone()).collect();
    let range = daily_range(hostnames);

    let source = Arc::new(NewRelicSource::new(
        SecretKey::new("newrelic_api_key", config.newrelic.api_key.clone()),
        config.newrelic.account_id.clone(),
    ));

    let report = run_report(&state.config_path, range, source).await?;
    *state.last_report.lock().unwrap() = Some(report.clone());
    Ok(report)
}

#[tauri::command]
pub async fn generate_weekly_report(state: State<'_, AppState>) -> Result<ReportDto, String> {
    let config = load_config(state.config_path.to_str().unwrap_or("config.toml"))
        .map_err(|e| e.to_string())?;

    if config.servers.is_empty() {
        return Err("config.toml 中 [[servers]] 列表不能为空".into());
    }
    let hostnames: Vec<String> = config.servers.iter().map(|s| s.hostname.clone()).collect();
    let range = weekly_range(hostnames);

    let source = Arc::new(NewRelicSource::new(
        SecretKey::new("newrelic_api_key", config.newrelic.api_key.clone()),
        config.newrelic.account_id.clone(),
    ));

    let report = run_report(&state.config_path, range, source).await?;
    *state.last_report.lock().unwrap() = Some(report.clone());
    Ok(report)
}

#[tauri::command]
pub async fn generate_custom_report(
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> Result<ReportDto, String> {
    let config = load_config(state.config_path.to_str().unwrap_or("config.toml"))
        .map_err(|e| e.to_string())?;

    if config.servers.is_empty() {
        return Err("config.toml 中 [[servers]] 列表不能为空".into());
    }
    let hostnames: Vec<String> = config.servers.iter().map(|s| s.hostname.clone()).collect();
    let range = parse_custom_range(&from, &to, hostnames).map_err(|e| e.to_string())?;

    let source = Arc::new(NewRelicSource::new(
        SecretKey::new("newrelic_api_key", config.newrelic.api_key.clone()),
        config.newrelic.account_id.clone(),
    ));

    let report = run_report(&state.config_path, range, source).await?;
    *state.last_report.lock().unwrap() = Some(report.clone());
    Ok(report)
}

#[tauri::command]
pub async fn generate_serilog_report(
    state: State<'_, AppState>,
    path: String,
    from: Option<String>,
    to: Option<String>,
) -> Result<ReportDto, String> {
    let range = serilog_range(from.as_deref(), to.as_deref()).map_err(|e| e.to_string())?;
    let source = Arc::new(SerilogFileSource::new(PathBuf::from(&path)));

    let report = run_report(&state.config_path, range, source).await?;
    *state.last_report.lock().unwrap() = Some(report.clone());
    Ok(report)
}
