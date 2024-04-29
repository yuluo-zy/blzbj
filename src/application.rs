use utils::chrono::{DateTime, Local};
use crate::settings::SettingsManager;

pub struct AppInfo {
    name: String,
    version: String,
    create_time: DateTime<Local>
}

pub struct AppStatus {
    cpu_percent: f64,
    memory_percent: f64,
    num_threads: u8
}

pub struct Application {
    settings_manager: SettingsManager,
    task_manager: 
}