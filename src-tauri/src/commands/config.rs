use tauri::State;

use ops_insight_core::Config;

use crate::state::AppState;

/// 返回原始 TOML 字符串（不解析 Keychain，避免密钥泄露到前端）
#[tauri::command]
pub fn load_config_cmd(state: State<'_, AppState>) -> Result<String, String> {
    std::fs::read_to_string(&state.config_path).map_err(|e| e.to_string())
}

/// 验证 TOML 语法后写入磁盘
#[tauri::command]
pub fn save_config_cmd(state: State<'_, AppState>, content: String) -> Result<(), String> {
    toml::from_str::<Config>(&content).map_err(|e| format!("TOML 格式错误: {e}"))?;
    std::fs::write(&state.config_path, content).map_err(|e| e.to_string())
}

/// 从 example 模板生成 config.toml
#[tauri::command]
pub fn init_config_cmd(state: State<'_, AppState>) -> Result<(), String> {
    if state.config_path.exists() {
        return Err("config.toml 已存在".into());
    }
    // 确保父目录存在
    if let Some(parent) = state.config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let template = include_str!("../../../ops-insight-core/config.example.toml");
    std::fs::write(&state.config_path, template).map_err(|e| e.to_string())
}

/// 返回配置文件路径（供前端展示）
#[tauri::command]
pub fn get_config_path(state: State<'_, AppState>) -> String {
    state.config_path.display().to_string()
}
