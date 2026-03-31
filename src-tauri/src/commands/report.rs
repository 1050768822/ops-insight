use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use ops_insight_core::{
    Config, GenerateReportUseCase, MarkdownWriter, NewRelicSource, ReportDto, SecretKey,
    SerilogFileSource, build_analyzer_with_provider, daily_range, load_config, parse_custom_range,
    serilog_range, weekly_range,
};

use crate::state::AppState;

const SUPPORTED_ANALYZERS: [&str; 4] = ["local", "claude", "openai", "deepseek"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedReportDto {
    pub analyzer: String,
    pub report: ReportDto,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateReportsResultDto {
    pub reports: Vec<GeneratedReportDto>,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerOptionsDto {
    pub supported: Vec<String>,
    pub default_selected: Vec<String>,
}

fn resolve_output_dir(path: &str) -> Result<PathBuf, String> {
    let output_dir = PathBuf::from(path);
    if output_dir.is_absolute() {
        return Ok(output_dir);
    }

    std::env::current_dir()
        .map(|cwd| cwd.join(output_dir))
        .map_err(|e| e.to_string())
}

fn normalize_analyzers(
    default_provider: &str,
    selected: Option<Vec<String>>,
) -> Result<Vec<String>, String> {
    let candidates = selected.unwrap_or_else(|| vec![default_provider.to_string()]);
    let mut normalized = Vec::new();

    for analyzer in candidates {
        if !SUPPORTED_ANALYZERS.contains(&analyzer.as_str()) {
            return Err(format!(
                "不支持的分析器 \"{analyzer}\"，支持: {}",
                SUPPORTED_ANALYZERS.join(", ")
            ));
        }

        if !normalized.iter().any(|item| item == &analyzer) {
            normalized.push(analyzer);
        }
    }

    if normalized.is_empty() {
        return Err("请至少选择一个分析器".into());
    }

    Ok(normalized)
}

async fn run_reports(
    config_path: &PathBuf,
    range: ops_insight_core::QueryRange,
    source: Arc<dyn ops_insight_core::DataSource>,
    analyzers: Option<Vec<String>>,
) -> Result<GenerateReportsResultDto, String> {
    let config = load_config(config_path.to_str().unwrap_or("config.toml"))
        .map_err(|e| e.to_string())?;
    let selected_analyzers = normalize_analyzers(&config.analyzer.provider, analyzers)?;
    let output_dir = resolve_output_dir(&config.output.output_dir)?;
    let mut reports = Vec::with_capacity(selected_analyzers.len());

    for analyzer_name in selected_analyzers {
        let analyzer =
            build_analyzer_with_provider(&config, &analyzer_name).map_err(|e| e.to_string())?;

        let writers: Vec<Arc<dyn ops_insight_core::ReportWriter>> = vec![Arc::new(
            MarkdownWriter::with_prefix(output_dir.clone(), analyzer_name.clone()),
        )];

        let report = GenerateReportUseCase::new(source.clone(), analyzer, writers)
            .execute(range.clone())
            .await
            .map_err(|e| e.to_string())?;

        reports.push(GeneratedReportDto {
            analyzer: analyzer_name,
            report,
        });
    }

    Ok(GenerateReportsResultDto {
        reports,
        output_dir: output_dir.display().to_string(),
    })
}

fn open_directory(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = std::process::Command::new("explorer");
        command.arg(path);
        command
    };
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut command = std::process::Command::new("xdg-open");

    #[cfg(any(target_os = "macos", all(not(target_os = "macos"), not(target_os = "windows"))))]
    command.arg(path);

    command.status().map_err(|e| e.to_string()).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err(format!("打开目录失败，退出码: {status}"))
        }
    })
}

#[tauri::command]
pub fn get_analyzer_options(state: State<'_, AppState>) -> Result<AnalyzerOptionsDto, String> {
    let content = std::fs::read_to_string(&state.config_path).map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&content).map_err(|e| e.to_string())?;
    let default_selected = normalize_analyzers(&config.analyzer.provider, None)?;

    Ok(AnalyzerOptionsDto {
        supported: SUPPORTED_ANALYZERS
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
        default_selected,
    })
}

#[tauri::command]
pub fn open_report_folder(path: String) -> Result<(), String> {
    let folder = PathBuf::from(path);
    if !folder.exists() {
        return Err(format!("目录不存在: {}", folder.display()));
    }
    if !folder.is_dir() {
        return Err(format!("路径不是目录: {}", folder.display()));
    }

    open_directory(&folder)
}

#[tauri::command]
pub async fn generate_daily_report(
    state: State<'_, AppState>,
    analyzers: Option<Vec<String>>,
) -> Result<GenerateReportsResultDto, String> {
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

    run_reports(&state.config_path, range, source, analyzers).await
}

#[tauri::command]
pub async fn generate_weekly_report(
    state: State<'_, AppState>,
    analyzers: Option<Vec<String>>,
) -> Result<GenerateReportsResultDto, String> {
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

    run_reports(&state.config_path, range, source, analyzers).await
}

#[tauri::command]
pub async fn generate_custom_report(
    state: State<'_, AppState>,
    from: String,
    to: String,
    analyzers: Option<Vec<String>>,
) -> Result<GenerateReportsResultDto, String> {
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

    run_reports(&state.config_path, range, source, analyzers).await
}

#[tauri::command]
pub async fn generate_serilog_report(
    state: State<'_, AppState>,
    path: String,
    from: Option<String>,
    to: Option<String>,
    analyzers: Option<Vec<String>>,
) -> Result<GenerateReportsResultDto, String> {
    let range = serilog_range(from.as_deref(), to.as_deref()).map_err(|e| e.to_string())?;
    let source = Arc::new(SerilogFileSource::new(PathBuf::from(&path)));

    run_reports(&state.config_path, range, source, analyzers).await
}
