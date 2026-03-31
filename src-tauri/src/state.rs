use std::path::PathBuf;

pub struct AppState {
    pub config_path: PathBuf,
}

impl AppState {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }
}
