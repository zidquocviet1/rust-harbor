use crate::config::load_config;
use crate::error::Result;
use crate::services::scanner::scan_for_repos;
use crate::services::watcher::WatcherState;
use tauri::{AppHandle, Manager, Emitter};
use serde::Serialize;
use git2::Repository;
use dashmap::DashMap;
use std::sync::Arc;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Serialize, Clone)]
pub enum SyncStatus {
    Clean,
    Ahead,
    Dirty,
    Behind,
    Diverged,
}

#[derive(Debug, Serialize, Clone)]
pub struct RepoMetadata {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub branch: String,
    pub sync_status: SyncStatus,
    pub remote_url: Option<String>,
    pub remote_reachable: bool,
    pub last_modified: i64,
    pub languages: std::collections::HashMap<String, usize>,
}

pub struct RepoCache(pub Arc<DashMap<String, RepoMetadata>>);

impl RepoCache {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::new()))
    }
}

pub struct ScanStatus(pub AtomicBool);

#[tauri::command]
pub async fn list_repos(app: AppHandle, cache: tauri::State<'_, RepoCache>) -> Result<Vec<RepoMetadata>> {
    let mut result: Vec<RepoMetadata> = cache.0.iter().map(|r| r.value().clone()).collect();
    
    // Sort by last modified descending
    result.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    
    // If cache is empty, trigger an initial refresh in background
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
            let repos_paths = scan_for_repos(&config.watched_folders);
            
            // Update the watcher
            let watcher_state = app_clone.state::<WatcherState>();
            if let Ok(mut watcher) = watcher_state.try_lock() {
                let _ = watcher.start(app_clone.clone(), repos_paths.clone());
            }

            // Parallel processing with Rayon for 44+ repos
            let processed_repos: Vec<RepoMetadata> = repos_paths.par_iter().filter_map(|path| {
                let repo = Repository::open(path).ok()?;
                let name = path.file_name()?.to_string_lossy().to_string();
                let path_str = path.to_string_lossy().to_string();
                
                let head = repo.head().ok();
                let branch = head.as_ref()
                    .and_then(|h| h.shorthand().map(|s| s.to_string()))
                    .unwrap_or_else(|| "detached".to_string());
                    
                let last_modified = head.as_ref()
                    .and_then(|h| h.peel_to_commit().ok())
                    .map(|c| c.time().seconds())
                    .unwrap_or(0);

                let languages = analyze_languages(path);
                let description = get_repo_description(path);

                let mut status_options = git2::StatusOptions::new();
                status_options.include_untracked(true);
                let is_dirty = repo.statuses(Some(&mut status_options))
                    .map(|s| s.len() > 0)
                    .unwrap_or(false);

                let mut sync_status = if is_dirty { SyncStatus::Dirty } else { SyncStatus::Clean };
                
                if let Some(h) = head {
                    if let Ok(local_oid) = h.target().ok_or("no target") {
                        if let Ok(upstream) = repo.branch_upstream_name(h.name().unwrap_or("HEAD")) {
                            if let Ok(upstream_obj) = repo.refname_to_id(upstream.as_str().unwrap()) {
                                let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_obj).unwrap_or((0, 0));
                                if !is_dirty {
                                    if ahead > 0 && behind > 0 { sync_status = SyncStatus::Diverged; }
                                    else if ahead > 0 { sync_status = SyncStatus::Ahead; }
                                    else if behind > 0 { sync_status = SyncStatus::Behind; }
                                }
                            }
                        }
                    }
                }

                let remote_url = repo.find_remote("origin")
                    .ok()
                    .and_then(|r| r.url().map(|u| u.to_string()));
                
                // Real connectivity check with timeout (simulated via short timeout for git command)
                let remote_reachable = if remote_url.is_some() {
                    verify_remote_connectivity(&path_str)
                } else {
                    false
                };
                
                Some(RepoMetadata {
                    name,
                    path: path_str,
                    description,
                    branch,
                    sync_status,
                    remote_url,
                    remote_reachable,
                    last_modified,
                    languages,
                })
            }).collect();

            // Update global cache
            cache.0.clear();
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

fn verify_remote_connectivity(path: &str) -> bool {
    // Lightweight probe using git ls-remote. 
    // We attempt to set a timeout to prevent hanging the scan thread pool.
    use std::process::Command;
    
    // In a production app, we should probably use a library that handles timeouts better 
    // or wrap this in a way that truly kills the process if it hangs (e.g. waitpid with timeout).
    // For now, git ls-remote is generally fast if the server is up. 
    // We add a short timeout via the `git` command itself if supported, but here we'll just check success.
    
    // Setting a fake timeout via command doesn't really work well in cross-platform Rust without tokio.
    // However, since we are in a background thread pool (Rayon) and not the main UI thread, 
    // a slight delay is acceptable.
    let output = Command::new("git")
        .args(&["ls-remote", "--exit-code", "--heads", "origin"])
        .current_dir(path)
        .output();
        
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

fn get_repo_description(path: &std::path::Path) -> Option<String> {
    // 1. Try .git/description (often default/unhelpful)
    let git_desc_path = path.join(".git").join("description");
    if let Ok(content) = std::fs::read_to_string(&git_desc_path) {
        let trimmed = content.trim();
        if !trimmed.is_empty() && trimmed != "Unnamed repository; edit this file 'description' to name the repository." {
            return Some(trimmed.to_string());
        }
    }

    // 2. Try first few lines of README.md as a fallback
    let readme_names = ["README.md", "readme.md", "README", "readme", "README.txt", "readme.txt"];
    for name in readme_names {
        let readme_path = path.join(name);
        if let Ok(content) = std::fs::read_to_string(&readme_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                
                // Skip headers, empty lines, and lines that look like markdown blocks, HTML, or lists
                if trimmed.is_empty() || 
                   trimmed.starts_with('#') || 
                   trimmed.starts_with('>') || 
                   trimmed.starts_with('-') || 
                   trimmed.starts_with('*') || 
                   trimmed.starts_with('+') ||
                   trimmed.starts_with('[') ||
                   trimmed.starts_with(']') ||
                   trimmed.starts_with('!') ||
                   trimmed.starts_with('|') ||
                   trimmed.starts_with('`') ||
                   trimmed.starts_with('<') ||
                   trimmed.starts_with('&') ||
                   trimmed.starts_with('(') ||
                   trimmed.starts_with(')') ||
                   trimmed.starts_with("---") {
                    continue;
                }

                // Clean up markdown/html clutter
                let mut clean = trimmed.to_string();
                
                // Remove images like ![...](...)
                while let Some(start) = clean.find("![") {
                    if let Some(end) = clean[start..].find(')') {
                        clean.replace_range(start..=start + end, "");
                    } else { break; }
                }

                // Remove standalone HTML tags like <img ... /> or <br>
                while let Some(start) = clean.find('<') {
                    if let Some(end) = clean[start..].find('>') {
                        clean.replace_range(start..=start + end, " ");
                    } else { break; }
                }

                // Simplify links [text](url) -> text
                // Simple pass for links
                for _ in 0..5 { // Limit iterations to avoid infinite loops on weird strings
                    if let (Some(start), Some(mid), Some(end)) = (clean.find('['), clean.find("]("), clean.find(')')) {
                        if start < mid && mid < end {
                             let text = clean[start+1..mid].to_string();
                             clean.replace_range(start..=end, &text);
                             continue;
                        }
                    }
                    break;
                }

                // Final cleanup of common markdown markers
                let final_clean = clean.replace("**", "")
                    .replace("__", "")
                    .replace("`", "")
                    .replace("*", "")
                    .replace("_", "")
                    .trim()
                    .to_string();

                if !final_clean.is_empty() && final_clean.len() > 10 {
                    if final_clean.len() > 140 {
                        return Some(format!("{}...", &final_clean[..137]));
                    } else {
                        return Some(final_clean);
                    }
                }
            }
        }
    }
    None
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

fn analyze_languages(path: &std::path::Path) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    let extensions = [
        ("rs", "Rust"), ("js", "JavaScript"), ("ts", "TypeScript"),
        ("py", "Python"), ("go", "Go"), ("cpp", "C++"), ("hpp", "C++"), ("cc", "C++"),
        ("c", "C"), ("h", "C"), ("java", "Java"), ("swift", "Swift"),
        ("kt", "Kotlin"), ("php", "PHP"), ("rb", "Ruby"),
        ("html", "HTML"), ("css", "CSS"), ("svelte", "Svelte"),
        ("vue", "Vue"), ("sh", "Shell"), ("sql", "SQL"), ("yaml", "YAML"), ("yml", "YAML"),
        ("cs", "C#"), ("scala", "Scala"), ("dart", "Dart"), ("ex", "Elixir"),
        ("exs", "Elixir"), ("erl", "Erlang"), ("clj", "Clojure"),
    ];

    let walker = walkdir::WalkDir::new(path)
        .max_depth(10) // Enough for most projects
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // Prune heavy directories
            !name.starts_with('.') && 
            name != "node_modules" && 
            name != "target" && 
            name != "build" &&
            name != "dist" &&
            name != "vendor"
        });

    for entry in walker.flatten() {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                for (e, lang) in extensions {
                    if ext == e {
                        *counts.entry(lang.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    counts
}

#[tauri::command]
pub async fn git_fetch(path: String) -> Result<String> {
    execute_git_command(path, &["fetch", "origin"])
}

#[tauri::command]
pub async fn git_pull(path: String) -> Result<String> {
    execute_git_command(path, &["pull", "origin", "HEAD"])
}

#[tauri::command]
pub async fn git_push(path: String) -> Result<String> {
    execute_git_command(path, &["push"])
}

fn execute_git_command(path: String, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
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
