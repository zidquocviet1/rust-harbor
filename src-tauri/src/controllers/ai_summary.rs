use crate::config::{load_ai_config, save_ai_config, AiConfig, AiConfigPublic};
use crate::error::Result;
use crate::services::ai::generate_summary;
use crate::services::database::DbPool;
use tauri::{AppHandle, State};

/// Returns the current AI configuration without exposing the raw API key.
#[tauri::command]
pub async fn get_ai_config(app: AppHandle) -> Result<AiConfigPublic> {
    let cfg = load_ai_config(&app)?;
    Ok(AiConfigPublic::from(&cfg))
}

/// Saves the AI configuration (including raw API key / OAuth token) to disk.
#[tauri::command]
pub async fn save_ai_config_cmd(
    app: AppHandle,
    provider: String,
    model: String,
    api_key: Option<String>,
    ollama_base_url: Option<String>,
    auth_method: Option<String>,
) -> Result<()> {
    let cfg = AiConfig {
        provider,
        model,
        api_key,
        ollama_base_url,
        auth_method: auth_method.unwrap_or_else(|| "api_key".to_string()),
    };
    save_ai_config(&app, &cfg)
}

/// Generates (or returns cached) AI summary for a pull history record.
///
/// - `pull_id`: ID of the pull_history entry to summarize.
/// - `force_regenerate`: If true, ignores the cached summary and regenerates.
/// - `selected_model`: Optional model override; if None, the configured model is used.
#[tauri::command]
pub async fn generate_pull_summary(
    app: AppHandle,
    db: State<'_, DbPool>,
    pull_id: i64,
    force_regenerate: bool,
    selected_model: Option<String>,
) -> Result<String> {
    let ai_config = load_ai_config(&app)?;
    generate_summary(pull_id, force_regenerate, &db, &ai_config, selected_model.as_deref()).await
}
