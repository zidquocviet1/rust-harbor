use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub watched_folders: Vec<String>,
    #[serde(default)]
    pub group_by_mode: Option<String>,
}

fn get_config_path(app_handle: &tauri::AppHandle) -> crate::error::Result<PathBuf> {
    use tauri::Manager;
    let mut path = app_handle.path().app_config_dir()
        .map_err(|e| crate::error::Error::SystemError(format!("Failed to get config dir: {}", e)))?;
    
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    
    path.push("config.json");
    Ok(path)
}

pub fn load_config(app_handle: &tauri::AppHandle) -> crate::error::Result<AppConfig> {
    let path = get_config_path(app_handle)?;
    
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    
    let content = fs::read_to_string(path)?;
    let config = serde_json::from_str(&content)
        .map_err(|e| crate::error::Error::ConfigError(format!("Failed to parse config: {}", e)))?;
    
    Ok(config)
}

pub fn save_config(app_handle: &tauri::AppHandle, config: &AppConfig) -> crate::error::Result<()> {
    let path = get_config_path(app_handle)?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| crate::error::Error::ConfigError(format!("Failed to serialize config: {}", e)))?;
    
    fs::write(path, content)?;
    Ok(())
}
