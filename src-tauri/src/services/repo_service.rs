use git2::Repository;
use std::path::Path;
use std::collections::HashMap;
use tauri::{AppHandle, Manager, Emitter};
use crate::models::repo::{SyncStatus, RepoMetadata};
use crate::controllers::repo::RepoCache;
use std::process::Command;

pub fn get_repo_metadata(path: &Path) -> Option<RepoMetadata> {
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
}

pub fn update_repo_cache(app: &AppHandle, path_str: &str) {
    let cache = app.state::<RepoCache>();
    let path = Path::new(path_str);
    if let Some(metadata) = get_repo_metadata(path) {
        cache.0.insert(path_str.to_string(), metadata);
        let _ = app.emit("repo-state-changed", ());
    }
}

pub fn verify_remote_connectivity(path: &str) -> bool {
    let output = Command::new("git")
        .args(&["ls-remote", "--exit-code", "--heads", "origin"])
        .current_dir(path)
        .output();
        
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

pub fn get_repo_description(path: &Path) -> Option<String> {
    let git_desc_path = path.join(".git").join("description");
    if let Ok(content) = std::fs::read_to_string(&git_desc_path) {
        let trimmed = content.trim();
        if !trimmed.is_empty() && trimmed != "Unnamed repository; edit this file 'description' to name the repository." {
            return Some(trimmed.to_string());
        }
    }

    let readme_names = ["README.md", "readme.md", "README", "readme", "README.txt", "readme.txt"];
    for name in readme_names {
        let readme_path = path.join(name);
        if let Ok(content) = std::fs::read_to_string(&readme_path) {
            for line in content.lines() {
                let trimmed = line.trim();
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

                let mut clean = trimmed.to_string();
                while let Some(start) = clean.find("![") {
                    if let Some(end) = clean[start..].find(')') {
                        clean.replace_range(start..=start + end, "");
                    } else { break; }
                }
                while let Some(start) = clean.find('<') {
                    if let Some(end) = clean[start..].find('>') {
                        clean.replace_range(start..=start + end, " ");
                    } else { break; }
                }
                for _ in 0..5 {
                    if let (Some(start), Some(mid), Some(end)) = (clean.find('['), clean.find("]("), clean.find(')')) {
                        if start < mid && mid < end {
                             let text = clean[start+1..mid].to_string();
                             clean.replace_range(start..=end, &text);
                             continue;
                        }
                    }
                    break;
                }
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

pub fn analyze_languages(path: &Path) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
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
        .max_depth(10)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
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
