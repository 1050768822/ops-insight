use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

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
    pub failures: Vec<AnalyzerFailureDto>,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerFailureDto {
    pub analyzer: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerOptionDto {
    pub id: String,
    pub enabled: bool,
    pub reason: Option<String>,
    pub selected_by_default: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerOptionsDto {
    pub analyzers: Vec<AnalyzerOptionDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportHistoryItemDto {
    pub file_name: String,
    pub path: String,
    pub modified_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportHistoryContentDto {
    pub file_name: String,
    pub path: String,
    pub content: String,
}

fn analyzer_unavailable_reason(config: &Config, analyzer: &str) -> Option<String> {
    match analyzer {
        "local" => None,
        "claude" => {
            if config.claude.api_key.trim().is_empty() {
                Some("缺少 [claude].api_key 配置".to_string())
            } else if config.claude.model.trim().is_empty() {
                Some("缺少 [claude].model 配置".to_string())
            } else {
                None
            }
        }
        "openai" => {
            if config.openai.api_key.trim().is_empty() {
                Some("缺少 [openai].api_key 配置".to_string())
            } else if config.openai.model.trim().is_empty() {
                Some("缺少 [openai].model 配置".to_string())
            } else {
                None
            }
        }
        "deepseek" => {
            if config.deepseek.api_key.trim().is_empty() {
                Some("缺少 [deepseek].api_key 配置".to_string())
            } else if config.deepseek.model.trim().is_empty() {
                Some("缺少 [deepseek].model 配置".to_string())
            } else {
                None
            }
        }
        other => Some(format!("不支持的分析器: {other}")),
    }
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

fn load_raw_config(config_path: &PathBuf) -> Result<Config, String> {
    let content = std::fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    toml::from_str(&content).map_err(|e| e.to_string())
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
    let mut failures = Vec::new();

    for analyzer_name in selected_analyzers {
        let analyzer = match build_analyzer_with_provider(&config, &analyzer_name) {
            Ok(analyzer) => analyzer,
            Err(err) => {
                failures.push(AnalyzerFailureDto {
                    analyzer: analyzer_name,
                    reason: err.to_string(),
                });
                continue;
            }
        };

        let writers: Vec<Arc<dyn ops_insight_core::ReportWriter>> = vec![Arc::new(
            MarkdownWriter::with_prefix(output_dir.clone(), analyzer_name.clone()),
        )];

        match GenerateReportUseCase::new(source.clone(), analyzer, writers)
            .execute(range.clone())
            .await
        {
            Ok(report) => {
                reports.push(GeneratedReportDto {
                    analyzer: analyzer_name,
                    report,
                });
            }
            Err(err) => {
                failures.push(AnalyzerFailureDto {
                    analyzer: analyzer_name,
                    reason: err.to_string(),
                });
            }
        }
    }

    if reports.is_empty() {
        let message = failures
            .iter()
            .map(|item| format!("{}: {}", item.analyzer, item.reason))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(message);
    }

    Ok(GenerateReportsResultDto {
        reports,
        failures,
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
    let config = load_raw_config(&state.config_path)?;
    let default_selected = normalize_analyzers(&config.analyzer.provider, None)?;
    let analyzers = SUPPORTED_ANALYZERS
        .iter()
        .map(|analyzer| {
            let reason = analyzer_unavailable_reason(&config, analyzer);
            let enabled = reason.is_none();
            let selected_by_default =
                enabled && default_selected.iter().any(|item| item == analyzer);

            AnalyzerOptionDto {
                id: (*analyzer).to_string(),
                enabled,
                reason,
                selected_by_default,
            }
        })
        .collect();

    Ok(AnalyzerOptionsDto { analyzers })
}

#[tauri::command]
pub fn list_report_history(state: State<'_, AppState>) -> Result<Vec<ReportHistoryItemDto>, String> {
    let config = load_raw_config(&state.config_path)?;
    let output_dir = resolve_output_dir(&config.output.output_dir)?;

    if !output_dir.exists() {
        return Ok(Vec::new());
    }

    let mut items = Vec::new();
    let entries = std::fs::read_dir(&output_dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }

        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs() as i64);

        items.push(ReportHistoryItemDto {
            file_name: entry.file_name().to_string_lossy().to_string(),
            path: path.display().to_string(),
            modified_at,
        });
    }

    items.sort_by(|left, right| right.modified_at.cmp(&left.modified_at));
    Ok(items)
}

#[tauri::command]
pub fn load_report_history(path: String) -> Result<ReportHistoryContentDto, String> {
    let report_path = PathBuf::from(&path);
    let content = std::fs::read_to_string(&report_path).map_err(|e| e.to_string())?;
    let file_name = report_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "无效的报告文件名".to_string())?
        .to_string();

    Ok(ReportHistoryContentDto {
        file_name,
        path,
        content,
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
