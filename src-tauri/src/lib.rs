mod commands;
mod state;

use state::AppState;
use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt().with_target(false).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let config_path = app
                .path()
                .app_config_dir()
                .map(|p| p.join("config.toml"))
                .unwrap_or_else(|_| std::path::PathBuf::from("config.toml"));
            app.manage(AppState::new(config_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::report::get_analyzer_options,
            commands::report::open_report_folder,
            commands::report::generate_daily_report,
            commands::report::generate_weekly_report,
            commands::report::generate_custom_report,
            commands::report::generate_serilog_report,
            commands::config::load_config_cmd,
            commands::config::save_config_cmd,
            commands::config::init_config_cmd,
            commands::config::get_config_path,
            commands::prompt::load_prompt_config,
            commands::prompt::save_prompt_config,
            commands::desensitize::get_builtin_labels,
            commands::desensitize::get_desensitize_config,
            commands::desensitize::save_desensitize_config,
            commands::desensitize::validate_pattern,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
