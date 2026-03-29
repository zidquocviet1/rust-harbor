use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;

// ── Window state ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
}

fn get_window_state_path(app_handle: &tauri::AppHandle) -> crate::error::Result<PathBuf> {
    use tauri::Manager;
    let mut path = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| crate::error::Error::SystemError(format!("Failed to get config dir: {}", e)))?;

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    path.push("window-state.json");
    Ok(path)
}

pub fn load_window_state(app_handle: &tauri::AppHandle) -> crate::error::Result<WindowState> {
    let path = get_window_state_path(app_handle)?;

    if !path.exists() {
        return Ok(WindowState { width: 1400, height: 900 });
    }

    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|e| crate::error::Error::ConfigError(format!("Failed to parse window state: {}", e)))
}

pub fn save_window_state(app_handle: &tauri::AppHandle, state: &WindowState) -> crate::error::Result<()> {
    let path = get_window_state_path(app_handle)?;
    let content = serde_json::to_string(state)
        .map_err(|e| crate::error::Error::ConfigError(format!("Failed to serialize window state: {}", e)))?;
    fs::write(path, content)?;
    Ok(())
}

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

// ── AI Configuration ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AiConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub ollama_base_url: Option<String>,
    /// Authentication method: "api_key" (default) or "oauth_token".
    /// Only Gemini supports "oauth_token"; all other providers use "api_key".
    #[serde(default)]
    pub auth_method: String,
}

/// Safe public view of AiConfig — never exposes the raw API key or token.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfigPublic {
    pub provider: String,
    pub model: String,
    pub ollama_base_url: Option<String>,
    pub has_api_key: bool,
    pub auth_method: String,
}

impl From<&AiConfig> for AiConfigPublic {
    fn from(cfg: &AiConfig) -> Self {
        Self {
            provider: cfg.provider.clone(),
            model: cfg.model.clone(),
            ollama_base_url: cfg.ollama_base_url.clone(),
            has_api_key: cfg.api_key.as_deref().map(|k| !k.is_empty()).unwrap_or(false),
            auth_method: if cfg.auth_method.is_empty() {
                "api_key".to_string()
            } else {
                cfg.auth_method.clone()
            },
        }
    }
}

fn get_ai_config_path(app_handle: &tauri::AppHandle) -> crate::error::Result<PathBuf> {
    use tauri::Manager;
    let mut path = app_handle.path().app_config_dir()
        .map_err(|e| crate::error::Error::SystemError(format!("Failed to get config dir: {}", e)))?;

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    path.push("ai-config.json");
    Ok(path)
}

pub fn load_ai_config(app_handle: &tauri::AppHandle) -> crate::error::Result<AiConfig> {
    let path = get_ai_config_path(app_handle)?;

    if !path.exists() {
        return Ok(AiConfig::default());
    }

    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|e| crate::error::Error::ConfigError(format!("Failed to parse AI config: {}", e)))
}

pub fn save_ai_config(app_handle: &tauri::AppHandle, config: &AiConfig) -> crate::error::Result<()> {
    let path = get_ai_config_path(app_handle)?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| crate::error::Error::ConfigError(format!("Failed to serialize AI config: {}", e)))?;
    fs::write(path, content)?;
    Ok(())
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
