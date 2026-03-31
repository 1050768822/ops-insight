use ops_insight_core::{Config, DesensitizeConfig, builtin_pattern_labels};
use tauri::State;

use crate::state::AppState;

/// 返回所有内置规则标签列表
#[tauri::command]
pub fn get_builtin_labels() -> Vec<String> {
    builtin_pattern_labels()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// 读取当前脱敏配置（从 config.toml 的 [desensitize] 段）
#[tauri::command]
pub fn get_desensitize_config(state: State<'_, AppState>) -> Result<DesensitizeConfig, String> {
    let content = std::fs::read_to_string(&state.config_path).map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config.desensitize)
}

/// 将脱敏配置写回 config.toml，只更新 [desensitize] 段，其余保持不变
#[tauri::command]
pub fn save_desensitize_config(
    state: State<'_, AppState>,
    config: DesensitizeConfig,
) -> Result<(), String> {
    let content = std::fs::read_to_string(&state.config_path).map_err(|e| e.to_string())?;
    let mut doc: toml::Value = toml::from_str(&content).map_err(|e| e.to_string())?;

    let new_section = toml::Value::try_from(&config).map_err(|e| e.to_string())?;

    if let toml::Value::Table(ref mut root) = doc {
        root.insert("desensitize".to_string(), new_section);
    }

    let updated = toml::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    std::fs::write(&state.config_path, updated).map_err(|e| e.to_string())
}

/// 验证正则表达式是否合法，返回错误信息或 null
#[tauri::command]
pub fn validate_pattern(pattern: String) -> Option<String> {
    regex::Regex::new(&pattern)
        .err()
        .map(|e| e.to_string())
}
