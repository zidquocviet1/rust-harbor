use crate::error::Result;
use crate::config::{AppConfig, load_config, save_config};
use tauri::AppHandle;

#[tauri::command]
pub async fn get_config(app: AppHandle) -> Result<AppConfig> {
    load_config(&app)
}

#[tauri::command]
pub async fn set_config(app: AppHandle, config: AppConfig) -> Result<()> {
    save_config(&app, &config)
}
