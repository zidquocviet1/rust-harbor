use crate::error::Result;
use crate::config::{AppConfig, load_config, save_config};
use crate::models::workspace::WorkspaceInsight;
use crate::controllers::repo::RepoCache;
use tauri::{AppHandle, Manager};
use std::path::Path;

#[tauri::command]
pub async fn get_config(app: AppHandle) -> Result<AppConfig> {
    load_config(&app)
}

#[tauri::command]
pub async fn set_config(app: AppHandle, config: AppConfig) -> Result<()> {
    save_config(&app, &config)
}

#[tauri::command]
pub async fn get_workspace_insights(app: AppHandle) -> Result<Vec<WorkspaceInsight>> {
    let config = load_config(&app)?;
    let cache = app.state::<RepoCache>();
    
    let mut insights = Vec::new();
    
    for folder in config.watched_folders {
        let repo_count = cache.0.iter()
            .filter(|r| r.value().path.starts_with(&folder))
            .count();
        
        // In a real app, we'd persist last scan per folder. 
        // For MVP, we derive from repo access or current time.
        let path_exists = Path::new(&folder).exists();
        
        insights.push(WorkspaceInsight {
            path: folder.clone(),
            repo_count,
            last_scan_time: Some(chrono::Utc::now().timestamp()), // Mock for now
            scan_status: if path_exists { "Synced".to_string() } else { "Warning".to_string() },
            error_details: if path_exists { None } else { Some("Directory not found or inaccessible".to_string()) },
        });
    }
    
    Ok(insights)
}

#[tauri::command]
pub async fn verify_git_path(path: String) -> Result<String> {
    let output = std::process::Command::new(&path)
        .arg("--version")
        .output()
        .map_err(|e| crate::error::Error::IoError(format!("Failed to execute '{}': {}", path, e)))?;
        
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(crate::error::Error::GitError(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}
