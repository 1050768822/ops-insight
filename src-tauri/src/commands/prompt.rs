use serde::{Deserialize, Serialize};
use tauri::State;

use ops_insight_core::PromptConfig;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptConfigDto {
    pub zh: String,
    pub en: String,
}

fn load_prompt_from_config(path: &std::path::Path) -> Result<PromptConfig, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config: ops_insight_core::Config = toml::from_str(&content).map_err(|e| e.to_string())?;
    Ok(config.prompt)
}

#[tauri::command]
pub fn load_prompt_config(state: State<'_, AppState>) -> Result<PromptConfigDto, String> {
    let prompt = load_prompt_from_config(&state.config_path)?;
    Ok(PromptConfigDto {
        zh: prompt.zh,
        en: prompt.en,
    })
}

#[tauri::command]
pub fn save_prompt_config(
    state: State<'_, AppState>,
    prompt: PromptConfigDto,
) -> Result<(), String> {
    let content = std::fs::read_to_string(&state.config_path).map_err(|e| e.to_string())?;
    let mut document = content
        .parse::<toml::Value>()
        .map_err(|e| format!("TOML 格式错误: {e}"))?;

    let prompt_value = toml::Value::try_from(PromptConfig {
        zh: prompt.zh,
        en: prompt.en,
    })
    .map_err(|e| e.to_string())?;

    let table = document
        .as_table_mut()
        .ok_or_else(|| "配置文件根节点必须是 table".to_string())?;
    table.insert("prompt".to_string(), prompt_value);

    let normalized =
        toml::from_str::<ops_insight_core::Config>(&document.to_string()).map_err(|e| e.to_string())?;
    let final_content = toml::to_string_pretty(&normalized).map_err(|e| e.to_string())?;
    std::fs::write(&state.config_path, final_content).map_err(|e| e.to_string())
}
