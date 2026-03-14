use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub watched_folders: Vec<String>,
    #[serde(default)]
    pub group_by_mode: Option<String>,
    #[serde(default = "default_exclusion_patterns")]
    pub exclusion_patterns: Vec<String>,
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    #[serde(default = "default_git_path")]
    pub git_path: String,
    #[serde(default = "default_auto_refresh")]
    pub auto_refresh: bool,
}

fn default_exclusion_patterns() -> Vec<String> {
    vec![
        "**/node_modules/**".to_string(),
        "**/target/**".to_string(),
        "**/.venv/**".to_string(),
        "**/.git/**".to_string(),
    ]
}

fn default_max_depth() -> u32 {
    5
}

fn default_git_path() -> String {
    "git".to_string()
}

fn default_auto_refresh() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            watched_folders: Vec::new(),
            group_by_mode: None,
            exclusion_patterns: default_exclusion_patterns(),
            max_depth: default_max_depth(),
            git_path: default_git_path(),
            auto_refresh: default_auto_refresh(),
        }
    }
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
    use tauri::Emitter;
    let path = get_config_path(app_handle)?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| crate::error::Error::ConfigError(format!("Failed to serialize config: {}", e)))?;
    
    fs::write(path, content)?;
    let _ = app_handle.emit("config-changed", ());
    Ok(())
}
