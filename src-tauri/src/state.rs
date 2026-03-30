use std::path::PathBuf;
use std::sync::Mutex;

use ops_insight_core::ReportDto;

pub struct AppState {
    pub config_path: PathBuf,
    pub last_report: Mutex<Option<ReportDto>>,
}

impl AppState {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            last_report: Mutex::new(None),
        }
    }
}
