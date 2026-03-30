use crate::config::{
    ai_configs_to_public, load_ai_config, load_ai_config_store, save_ai_config_store,
    AiConfig, AiConfigPublic, AiConfigsPublic, ProviderConfig,
};
use crate::error::Result;
use crate::services::ai::generate_summary;
use crate::services::database::DbPool;
use tauri::{AppHandle, State};

/// Returns the active provider's config (legacy — used by AiSummaryPanel).
#[tauri::command]
pub async fn get_ai_config(app: AppHandle) -> Result<AiConfigPublic> {
    let cfg = load_ai_config(&app)?;
    Ok(AiConfigPublic {
        provider: cfg.provider.clone(),
        model: cfg.model.clone(),
        ollama_base_url: cfg.ollama_base_url.clone(),
        has_api_key: cfg.api_key.as_deref().map(|k| !k.is_empty()).unwrap_or(false),
        auth_method: if cfg.auth_method.is_empty() { "api_key".to_string() } else { cfg.auth_method },
    })
}

/// Returns all per-provider configs (without raw keys) and the active provider.
#[tauri::command]
pub async fn get_ai_configs(app: AppHandle) -> Result<AiConfigsPublic> {
    let store = load_ai_config_store(&app)?;
    Ok(ai_configs_to_public(&store))
}

/// Saves (or updates) the config for a single provider. Also sets it as active.
#[tauri::command]
pub async fn save_provider_config(
    app: AppHandle,
    provider: String,
    model: String,
    api_key: Option<String>,
    ollama_base_url: Option<String>,
    auth_method: Option<String>,
) -> Result<()> {
    let mut store = load_ai_config_store(&app)?;
    let existing = store.providers.get(&provider);
    // Preserve existing api_key if caller sends None (meaning "don't change")
    let resolved_key = match &api_key {
        Some(k) if !k.is_empty() => Some(k.clone()),
        Some(_) => None,
        None => existing.and_then(|e| e.api_key.clone()),
    };
    store.providers.insert(provider.clone(), ProviderConfig {
        model,
        api_key: resolved_key,
        ollama_base_url,
        auth_method: auth_method.unwrap_or_else(|| "api_key".to_string()),
    });
    store.active_provider = provider;
    save_ai_config_store(&app, &store)
}

/// Sets which provider is used by default for summary generation (without changing its config).
#[tauri::command]
pub async fn set_active_provider(app: AppHandle, provider: String) -> Result<()> {
    let mut store = load_ai_config_store(&app)?;
    store.active_provider = provider;
    save_ai_config_store(&app, &store)
}

/// Removes a provider's saved config entirely.
#[tauri::command]
pub async fn clear_provider_config(app: AppHandle, provider: String) -> Result<()> {
    let mut store = load_ai_config_store(&app)?;
    store.providers.remove(&provider);
    if store.active_provider == provider {
        store.active_provider = store.providers.keys().next().cloned().unwrap_or_default();
    }
    save_ai_config_store(&app, &store)
}

/// Legacy single-provider save — kept so old code paths don't break.
#[tauri::command]
pub async fn save_ai_config_cmd(
    app: AppHandle,
    provider: String,
    model: String,
    api_key: Option<String>,
    ollama_base_url: Option<String>,
    auth_method: Option<String>,
) -> Result<()> {
    save_provider_config(app, provider, model, api_key, ollama_base_url, auth_method).await
}

/// Generates (or returns cached) AI summary for a pull history record.
/// If `selected_provider` is given, uses that provider's stored config instead of the active one.
#[tauri::command]
pub async fn generate_pull_summary(
    app: AppHandle,
    db: State<'_, DbPool>,
    pull_id: i64,
    force_regenerate: bool,
    selected_model: Option<String>,
    selected_provider: Option<String>,
) -> Result<String> {
    let ai_config = if let Some(ref prov) = selected_provider {
        let store = load_ai_config_store(&app)?;
        if let Some(pc) = store.providers.get(prov) {
            AiConfig {
                provider: prov.clone(),
                model: pc.model.clone(),
                api_key: pc.api_key.clone(),
                ollama_base_url: pc.ollama_base_url.clone(),
                auth_method: pc.auth_method.clone(),
            }
        } else {
            load_ai_config(&app)?
        }
    } else {
        load_ai_config(&app)?
    };
    generate_summary(pull_id, force_regenerate, &db, &ai_config, selected_model.as_deref()).await
}
