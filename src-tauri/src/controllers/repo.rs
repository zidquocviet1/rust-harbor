use crate::config::load_config;
use crate::error::Result;
use crate::services::scanner::scan_for_repos;
use crate::services::watcher::WatcherState;
use crate::services::repo_service::{get_repo_metadata_local, update_repo_cache, verify_remote_connectivity};
use crate::services::database::{DbPool, cleanup_orphaned_tags, batch_fetch_repo_tags, load_repositories, save_repositories, clear_repositories, insert_pull_history};
use crate::models::repo::RepoMetadata;
use crate::models::editor::EditorInfo;
use crate::models::pull_history::{NewPullHistory, NewPullHistoryFile, PullResult};
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

            // --- Phase 1: fast local metadata, no network calls ---
            let git_path_ref = &config.git_path;
            let mut processed_repos: Vec<RepoMetadata> = repos_paths.par_iter().filter_map(|path| {
                get_repo_metadata_local(path, git_path_ref, false)
            }).collect();

            // Batch-fetch tags from SQLite and merge into metadata
            let db = app_clone.state::<DbPool>();
            if let Ok(conn) = db.0.lock() {
                if let Ok(tag_map) = batch_fetch_repo_tags(&conn) {
                    for repo in &mut processed_repos {
                        if let Some(tags) = tag_map.get(&repo.path) {
                            repo.tags = tags.clone();
                        }
                    }
                }
                let valid_paths: Vec<String> = processed_repos.iter().map(|r| r.path.clone()).collect();
                let _ = cleanup_orphaned_tags(&conn, &valid_paths);
            }

            cache.0.clear();
            if let Ok(conn) = db.0.lock() {
                let _ = clear_repositories(&conn);
                let _ = save_repositories(&conn, &processed_repos);
            }
            for repo in &processed_repos {
                cache.0.insert(repo.path.clone(), repo.clone());
            }
            Ok(())
        })();

        if let Err(e) = result {
            eprintln!("Background refresh failed: {:?}", e);
        }

        // Phase 1 done — UI can display repos immediately
        let status = app_clone.state::<ScanStatus>();
        status.0.store(false, Ordering::SeqCst);
        let _ = app_clone.emit("scan-completed", ());

        // --- Phase 2: background connectivity check (non-blocking for the UI) ---
        let app_phase2 = app_clone.clone();
        tokio::task::spawn_blocking(move || {
            let cache = app_phase2.state::<RepoCache>();
            let db = app_phase2.state::<DbPool>();
            let git_path = load_config(&app_phase2)
                .map(|c| c.git_path)
                .unwrap_or_else(|_| "git".to_string());

            // Collect repos that have a remote URL — only these need a connectivity check
            let repos_with_remote: Vec<String> = cache.0.iter()
                .filter(|r| r.value().remote_url.is_some())
                .map(|r| r.key().clone())
                .collect();

            if repos_with_remote.is_empty() {
                return;
            }

            // Tell the UI which repos are about to be checked
            let _ = app_phase2.emit("connectivity-check-started", &repos_with_remote);

            repos_with_remote.par_iter().for_each(|path_str| {
                let reachable = verify_remote_connectivity(path_str, &git_path);
                if let Some(mut entry) = cache.0.get_mut(path_str) {
                    entry.remote_reachable = reachable;
                }
            });

            // Persist all updated reachability values in one batch
            let updated: Vec<RepoMetadata> = repos_with_remote.iter()
                .filter_map(|p| cache.0.get(p).map(|e| e.clone()))
                .collect();
            if let Ok(conn) = db.0.lock() {
                let _ = save_repositories(&conn, &updated);
            }

            // Signal completion — single event avoids N re-renders in the UI
            let _ = app_phase2.emit("connectivity-check-completed", ());
            let _ = app_phase2.emit("repo-state-changed", ());
        });
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
pub async fn git_pull(app: AppHandle, path: String) -> Result<PullResult> {
    let config = load_config(&app)?;
    let git = &config.git_path;

    // 3.1 Capture HEAD SHA before pull
    let commit_before = execute_git_command(path.clone(), &["rev-parse", "HEAD"], git)
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Run the pull
    let output = execute_git_command(path.clone(), &["pull", "origin", "HEAD"], git)?;

    // 3.2 Capture HEAD SHA after pull
    let commit_after = execute_git_command(path.clone(), &["rev-parse", "HEAD"], git)
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    update_repo_cache(&app, &path);

    // 3.3 If SHAs are identical, no new commits — skip history
    if commit_before.is_empty() || commit_before == commit_after {
        return Ok(PullResult { output, history_id: None });
    }

    // 3.4 Run --numstat to get per-file stats
    let diff_range = format!("{}..{}", commit_before, commit_after);
    let numstat_output = execute_git_command(
        path.clone(),
        &["diff", &diff_range, "--numstat"],
        git,
    ).unwrap_or_default();

    // 3.5 Run full unified diff
    let full_diff = execute_git_command(
        path.clone(),
        &["diff", &diff_range, "--unified=3"],
        git,
    ).unwrap_or_default();

    // 3.4 Parse numstat lines into file stat records
    // Format: "<additions>\t<deletions>\t<filepath>"  (binary: "-\t-\t<filepath>")
    let file_stats: Vec<(String, i64, i64)> = numstat_output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() < 3 { return None; }
            let (add_str, del_str, filepath) = (parts[0], parts[1], parts[2]);
            // 3.6 Handle binary files
            let additions: i64 = add_str.parse().unwrap_or(0);
            let deletions: i64 = del_str.parse().unwrap_or(0);
            Some((filepath.to_string(), additions, deletions))
        })
        .collect();

    // Split full diff into per-file sections by "diff --git" headers
    let file_diffs = split_diff_by_file(&full_diff);

    // 3.8 Determine change_type from numstat / diff headers
    // "new file mode" → added, "deleted file mode" → deleted, "rename" → renamed, else → modified
    let change_type_map = detect_change_types(&full_diff);

    // 3.7 Build NewPullHistoryFile entries, capping diff_content at 500KB
    const MAX_DIFF_BYTES: usize = 500 * 1024;
    let files: Vec<NewPullHistoryFile> = file_stats
        .into_iter()
        .map(|(file_path, additions, deletions)| {
            let change_type = change_type_map
                .get(&file_path)
                .cloned()
                .unwrap_or_else(|| "modified".to_string());

            let raw_diff = file_diffs.get(&file_path).cloned().unwrap_or_default();
            let diff_content = if change_type == "binary" || (additions == 0 && deletions == 0 && raw_diff.is_empty()) {
                "[binary file]".to_string()
            } else if raw_diff.len() > MAX_DIFF_BYTES {
                format!("{}\n[diff truncated]", &raw_diff[..MAX_DIFF_BYTES])
            } else {
                raw_diff
            };

            NewPullHistoryFile {
                file_path,
                change_type,
                additions,
                deletions,
                diff_content,
            }
        })
        .collect();

    // 3.9 Determine branch name
    let branch = execute_git_command(path.clone(), &["rev-parse", "--abbrev-ref", "HEAD"], git)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Determine repo name from path
    let repo_name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let new_history = NewPullHistory {
        repo_path: path.clone(),
        repo_name,
        branch,
        pulled_at: chrono::Utc::now().timestamp(),
        commit_before,
        commit_after,
        files,
    };

    // 3.9 Persist — best-effort, do not fail the pull on error
    let history_id = if let Some(db) = app.try_state::<DbPool>() {
        match db.0.lock() {
            Ok(conn) => match insert_pull_history(&conn, &new_history) {
                Ok(id) => Some(id),
                Err(e) => {
                    eprintln!("[pull-history] Failed to insert history: {:?}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("[pull-history] Failed to lock DB: {:?}", e);
                None
            }
        }
    } else {
        None
    };

    Ok(PullResult { output, history_id })
}

/// Splits a unified diff string into a map of file_path → diff_section.
fn split_diff_by_file(full_diff: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut current_file: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();

    for line in full_diff.lines() {
        if line.starts_with("diff --git ") {
            // Flush previous file
            if let Some(file) = current_file.take() {
                map.insert(file, current_lines.join("\n"));
            }
            current_lines.clear();
            // Parse "diff --git a/<path> b/<path>"
            // Take the b/ path (after-pull version)
            if let Some(b_part) = line.split(" b/").nth(1) {
                current_file = Some(b_part.to_string());
            }
        }
        current_lines.push(line);
    }
    if let Some(file) = current_file {
        map.insert(file, current_lines.join("\n"));
    }
    map
}

/// Detects change_type for each file from diff headers.
fn detect_change_types(full_diff: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut current_file: Option<String> = None;
    let mut change_type = "modified".to_string();

    for line in full_diff.lines() {
        if line.starts_with("diff --git ") {
            if let Some(file) = current_file.take() {
                map.insert(file, change_type.clone());
            }
            change_type = "modified".to_string();
            if let Some(b_part) = line.split(" b/").nth(1) {
                current_file = Some(b_part.to_string());
            }
        } else if line.starts_with("new file mode") {
            change_type = "added".to_string();
        } else if line.starts_with("deleted file mode") {
            change_type = "deleted".to_string();
        } else if line.starts_with("rename ") || line.starts_with("similarity index") {
            change_type = "renamed".to_string();
        } else if line.contains("Binary files") {
            change_type = "binary".to_string();
        }
    }
    if let Some(file) = current_file {
        map.insert(file, change_type);
    }
    map
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
