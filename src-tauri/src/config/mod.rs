use serde::{Serialize, Deserialize};
use std::collections::HashMap;
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

/// Config for a single provider (stored in the per-provider map).
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProviderConfig {
    pub model: String,
    pub api_key: Option<String>,
    pub ollama_base_url: Option<String>,
    #[serde(default)]
    pub auth_method: String,
}

/// Multi-provider config store written to disk.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AiConfigStore {
    #[serde(default)]
    pub active_provider: String,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// Flat config view used by the AI service internally.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AiConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub ollama_base_url: Option<String>,
    #[serde(default)]
    pub auth_method: String,
}

/// Public view of a single provider (includes raw key for local display).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfigPublic {
    pub model: String,
    pub api_key: Option<String>,
    pub has_api_key: bool,
    pub ollama_base_url: Option<String>,
    pub auth_method: String,
}

/// Full public view returned to the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfigsPublic {
    pub active_provider: String,
    pub providers: HashMap<String, ProviderConfigPublic>,
}

/// Legacy single-provider public view (kept for AiSummaryPanel compatibility).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiConfigPublic {
    pub provider: String,
    pub model: String,
    pub ollama_base_url: Option<String>,
    pub has_api_key: bool,
    pub auth_method: String,
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

/// Load the multi-provider store, migrating the old single-provider format if needed.
pub fn load_ai_config_store(app_handle: &tauri::AppHandle) -> crate::error::Result<AiConfigStore> {
    let path = get_ai_config_path(app_handle)?;
    if !path.exists() {
        return Ok(AiConfigStore::default());
    }
    let content = fs::read_to_string(&path)?;
    // Try new format first
    if let Ok(store) = serde_json::from_str::<AiConfigStore>(&content) {
        // New format has `providers` key; old format has `provider` key at root
        if store.providers.is_empty() {
            // Could be empty new-format OR old format — check for `provider` field
            if let Ok(old) = serde_json::from_str::<AiConfig>(&content) {
                if !old.provider.is_empty() {
                    return Ok(migrate_old_config(old));
                }
            }
        }
        return Ok(store);
    }
    // Fall back to old format
    if let Ok(old) = serde_json::from_str::<AiConfig>(&content) {
        return Ok(migrate_old_config(old));
    }
    Ok(AiConfigStore::default())
}

fn migrate_old_config(old: AiConfig) -> AiConfigStore {
    let mut providers = HashMap::new();
    if !old.provider.is_empty() {
        providers.insert(old.provider.clone(), ProviderConfig {
            model: old.model,
            api_key: old.api_key,
            ollama_base_url: old.ollama_base_url,
            auth_method: old.auth_method,
        });
    }
    AiConfigStore { active_provider: old.provider, providers }
}

pub fn save_ai_config_store(app_handle: &tauri::AppHandle, store: &AiConfigStore) -> crate::error::Result<()> {
    let path = get_ai_config_path(app_handle)?;
    let content = serde_json::to_string_pretty(store)
        .map_err(|e| crate::error::Error::ConfigError(format!("Failed to serialize AI config: {}", e)))?;
    fs::write(path, content)?;
    Ok(())
}

/// Returns the active provider's config as a flat AiConfig for use by the AI service.
pub fn load_ai_config(app_handle: &tauri::AppHandle) -> crate::error::Result<AiConfig> {
    let store = load_ai_config_store(app_handle)?;
    let provider = store.active_provider.clone();
    if let Some(pc) = store.providers.get(&provider) {
        Ok(AiConfig {
            provider,
            model: pc.model.clone(),
            api_key: pc.api_key.clone(),
            ollama_base_url: pc.ollama_base_url.clone(),
            auth_method: pc.auth_method.clone(),
        })
    } else {
        Ok(AiConfig { provider, ..Default::default() })
    }
}

pub fn ai_configs_to_public(store: &AiConfigStore) -> AiConfigsPublic {
    AiConfigsPublic {
        active_provider: store.active_provider.clone(),
        providers: store.providers.iter().map(|(id, pc)| {
            (id.clone(), ProviderConfigPublic {
                model: pc.model.clone(),
                api_key: pc.api_key.clone(),
                has_api_key: pc.api_key.as_deref().map(|k| !k.is_empty()).unwrap_or(false),
                ollama_base_url: pc.ollama_base_url.clone(),
                auth_method: if pc.auth_method.is_empty() { "api_key".to_string() } else { pc.auth_method.clone() },
            })
        }).collect(),
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
