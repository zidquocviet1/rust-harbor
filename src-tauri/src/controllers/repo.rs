use crate::config::load_config;
use crate::error::Result;
use crate::services::scanner::scan_for_repos;
use crate::services::watcher::WatcherState;
use crate::services::repo_service::{get_repo_metadata, update_repo_cache};
use crate::services::database::{DbPool, cleanup_orphaned_tags, batch_fetch_repo_tags, load_repositories, save_repositories, clear_repositories};
use crate::models::repo::RepoMetadata;
use crate::models::editor::EditorInfo;
use crate::services::editor::{get_installed_editors as check_editors, open_path_in_editor};
use tauri::{AppHandle, Manager, Emitter};
use serde::Serialize;
use dashmap::DashMap;
use std::sync::Arc;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct RepoCache(pub Arc<DashMap<String, RepoMetadata>>);

impl RepoCache {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }
}

pub struct ScanStatus(pub AtomicBool);

#[tauri::command]
pub async fn list_repos(
    app: AppHandle,
    cache: tauri::State<'_, RepoCache>,
    db: tauri::State<'_, DbPool>,
) -> Result<Vec<RepoMetadata>> {
    // 1. If cache is empty, try loading from SQLite first
    if cache.0.is_empty() {
        if let Ok(conn) = db.0.lock() {
            if let Ok(db_repos) = load_repositories(&conn) {
                for repo in db_repos {
                    cache.0.insert(repo.path.clone(), repo);
                }
            }
        }
    }

    let mut result: Vec<RepoMetadata> = cache.0.iter().map(|r| r.value().clone()).collect();

    // 2. Merge latest tag assignments from SQLite
    if let Ok(conn) = db.0.lock() {
        if let Ok(tag_map) = batch_fetch_repo_tags(&conn) {
            for repo in &mut result {
                if let Some(tags) = tag_map.get(&repo.path) {
                    repo.tags = tags.clone();
                } else {
                    repo.tags = vec![];
                }
            }
        }
    }
    
    // Sort by last modified descending
    result.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    
    // 3. If STILL empty (first run), trigger a scan
    if result.is_empty() {
        let app_clone = app.clone();
        tokio::spawn(async move {
            let _ = refresh_repos(app_clone).await;
        });
    }
    
    Ok(result)
}

#[tauri::command]
pub async fn is_scanning(status: tauri::State<'_, ScanStatus>) -> Result<bool> {
    Ok(status.0.load(Ordering::Relaxed))
}

#[tauri::command]
pub async fn refresh_repos(app: AppHandle) -> Result<()> {
    let status = app.state::<ScanStatus>();
    
    // Skip if already scanning
    if status.0.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let app_clone = app.clone();
    
    // Run in background and return immediately
    tokio::spawn(async move {
        let _ = app_clone.emit("scan-started", ());
        
        let result = (|| -> Result<()> {
            let config = load_config(&app_clone)?;
            let cache = app_clone.state::<RepoCache>();
            let repos_paths = scan_for_repos(&config);
            
            // Update the watcher
            let watcher_state = app_clone.state::<WatcherState>();
            if let Ok(mut watcher) = watcher_state.try_lock() {
                let _ = watcher.start(app_clone.clone(), repos_paths.clone());
            }

            // Parallel processing with Rayon for repos
            let git_path_ref = &config.git_path;
            let mut processed_repos: Vec<RepoMetadata> = repos_paths.par_iter().filter_map(|path| {
                get_repo_metadata(path, git_path_ref)
            }).collect();

            // Batch-fetch tags from SQLite and merge into metadata
            let db = app_clone.state::<DbPool>();
            if let Ok(conn) = db.0.lock() {
                // Fetch all tag assignments
                if let Ok(tag_map) = batch_fetch_repo_tags(&conn) {
                    for repo in &mut processed_repos {
                        if let Some(tags) = tag_map.get(&repo.path) {
                            repo.tags = tags.clone();
                        }
                    }
                }

                // Cleanup orphaned repo_tags entries
                let valid_paths: Vec<String> = processed_repos.iter().map(|r| r.path.clone()).collect();
                let _ = cleanup_orphaned_tags(&conn, &valid_paths);
            }

            cache.0.clear();
            if let Ok(conn) = db.0.lock() {
                let _ = clear_repositories(&conn);
                let _ = save_repositories(&conn, &processed_repos);
            }

            for repo in processed_repos {
                cache.0.insert(repo.path.clone(), repo);
            }
            Ok(())
        })();

        if let Err(e) = result {
            eprintln!("Background refresh failed: {:?}", e);
        }

        let status = app_clone.state::<ScanStatus>();
        status.0.store(false, Ordering::SeqCst);
        let _ = app_clone.emit("scan-completed", ());
    });

    Ok(())
}

#[derive(Debug, Serialize, Clone)]
pub struct ReadmeResponse {
    pub html: String,
    pub raw: String,
}

#[tauri::command]
pub async fn get_repo_readme(path: String) -> Result<ReadmeResponse> {
    let base_path = std::path::Path::new(&path);
    let readme_names = ["README.md", "readme.md", "README", "readme", "README.txt", "readme.txt"];
    
    for name in readme_names {
        let readme_path = base_path.join(name);
        if readme_path.exists() {
            let content = std::fs::read_to_string(readme_path)
                .map_err(|e| crate::error::Error::IoError(e.to_string()))?;
            
            // Use Comrak for robust GFM parsing
            let mut options = comrak::Options::default();
            options.extension.table = true;
            options.extension.strikethrough = true;
            options.extension.tasklist = true;
            options.extension.autolink = true;
            options.extension.description_lists = true;
            options.extension.footnotes = true;
            options.extension.front_matter_delimiter = Some("---".to_string());
            options.render.unsafe_ = true; 

            let html = comrak::markdown_to_html(&content, &options);
            return Ok(ReadmeResponse {
                html,
                raw: content,
            });
        }
    }
    
    Err(crate::error::Error::IoError("README not found".to_string()))
}

#[tauri::command]
pub async fn git_fetch(app: AppHandle, path: String) -> Result<String> {
    let config = load_config(&app)?;
    let result = execute_git_command(path.clone(), &["fetch", "origin"], &config.git_path)?;
    update_repo_cache(&app, &path);
    Ok(result)
}

#[tauri::command]
pub async fn git_pull(app: AppHandle, path: String) -> Result<String> {
    let config = load_config(&app)?;
    let result = execute_git_command(path.clone(), &["pull", "origin", "HEAD"], &config.git_path)?;
    update_repo_cache(&app, &path);
    Ok(result)
}

#[tauri::command]
pub async fn git_push(app: AppHandle, path: String) -> Result<String> {
    let config = load_config(&app)?;
    let result = execute_git_command(path.clone(), &["push"], &config.git_path)?;
    update_repo_cache(&app, &path);
    Ok(result)
}

fn execute_git_command(path: String, args: &[&str], git_path: &str) -> Result<String> {
    let output = std::process::Command::new(git_path)
        .args(args)
        .current_dir(path)
        .output()
        .map_err(|e| crate::error::Error::IoError(e.to_string()))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(crate::error::Error::GitError(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

#[tauri::command]
pub async fn get_installed_editors() -> Result<Vec<EditorInfo>> {
    Ok(check_editors())
}

#[tauri::command]
pub async fn open_in_editor(editor_id: String, path: String) -> Result<()> {
    open_path_in_editor(&editor_id, &path).map_err(|e| crate::error::Error::IoError(e))
}
