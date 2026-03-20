use git2::Repository;
use std::path::Path;
use std::collections::HashMap;
use tauri::{AppHandle, Manager, Emitter};
use crate::models::repo::{SyncStatus, RepoMetadata};
use crate::controllers::repo::RepoCache;
use crate::services::database::{DbPool, batch_fetch_repo_tags};
use std::process::Command;

/// Extract the integer after `keyword` in a string (e.g. "ahead 3" → 3).
fn extract_count(s: &str, keyword: &str) -> usize {
    s.find(keyword)
        .and_then(|pos| {
            s[pos + keyword.len()..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()
        })
        .unwrap_or(0)
}

/// Compute sync status using `git status --porcelain --branch`.
///
/// This is the authoritative source — it matches exactly what the user sees
/// in their terminal. It is a local-only operation (uses cached remote refs,
/// no network call) and typically completes in < 10 ms.
///
/// Priority: Conflict > Dirty > Diverged > Ahead > Behind > NoUpstream > Clean
fn compute_sync_status(repo: &Repository, path: &Path, git_path: &str) -> SyncStatus {
    // 1. In-progress operations detected via .git sentinel files
    let git_dir = repo.path();
    if git_dir.join("MERGE_HEAD").exists()
        || git_dir.join("CHERRY_PICK_HEAD").exists()
        || git_dir.join("REBASE_MERGE").exists()
        || git_dir.join("REBASE_APPLY").exists()
    {
        return SyncStatus::Conflict;
    }

    // 2. `git status --porcelain --branch` output format:
    //    Line 1: ## <branch>...<upstream> [ahead N][, behind M]   (or ## No commits yet, etc.)
    //    Rest:   XY filename  where X=index status, Y=worktree status
    //            '??' = untracked, '!!' = ignored
    let output = Command::new(git_path)
        .args(["status", "--porcelain", "--branch"])
        .current_dir(path)
        .output();

    let stdout = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => return SyncStatus::Clean, // git unavailable / bare repo — show clean
    };

    let mut lines = stdout.lines();
    let branch_line = match lines.next() {
        Some(l) => l,
        None => return SyncStatus::Clean,
    };

    // Unborn branch (repo initialised but no commits yet)
    if branch_line.starts_with("## No commits yet") || branch_line == "## HEAD" {
        return SyncStatus::NoUpstream;
    }

    // Detached HEAD — no upstream concept, just report dirty/clean
    if branch_line.starts_with("## HEAD (no branch)") {
        let is_dirty = lines.any(|l| l.len() >= 2 && !l.starts_with("??") && !l.starts_with("!!"));
        return if is_dirty { SyncStatus::Dirty } else { SyncStatus::Clean };
    }

    let has_upstream = branch_line.contains("...");
    // [gone] means the remote tracking branch was deleted on the remote
    let upstream_gone = branch_line.contains("[gone]");
    let ahead  = extract_count(branch_line, "ahead ");
    let behind = extract_count(branch_line, "behind ");

    // Dirty = any file line that is not untracked (??) or ignored (!!)
    // This covers: staged new files (A ), staged edits (M ), unstaged edits ( M),
    // deletions, renames — but NOT untracked files, which don't need a commit.
    let is_dirty = lines.any(|l| l.len() >= 2 && !l.starts_with("??") && !l.starts_with("!!"));

    if is_dirty {
        return SyncStatus::Dirty;
    }

    if !has_upstream || upstream_gone {
        return SyncStatus::NoUpstream;
    }

    match (ahead, behind) {
        (a, b) if a > 0 && b > 0 => SyncStatus::Diverged,
        (a, _) if a > 0          => SyncStatus::Ahead,
        (_, b) if b > 0          => SyncStatus::Behind,
        _                         => SyncStatus::Clean,
    }
}

pub fn get_repo_metadata(path: &Path, git_path: &str) -> Option<RepoMetadata> {
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
    let sync_status = compute_sync_status(&repo, path, git_path);

    let remote_url = repo.find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|u| u.to_string()));

    let remote_reachable = if remote_url.is_some() {
        verify_remote_connectivity(&path_str, git_path)
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
        tags: vec![],
    })
}

pub fn update_repo_cache(app: &AppHandle, path_str: &str) {
    let cache = app.state::<RepoCache>();
    let path = Path::new(path_str);

    let git_path = crate::config::load_config(app)
        .map(|c| c.git_path)
        .unwrap_or_else(|_| "git".to_string());

    if let Some(mut metadata) = get_repo_metadata(path, &git_path) {
        if let Ok(conn) = app.state::<DbPool>().0.lock() {
            if let Ok(tag_map) = batch_fetch_repo_tags(&conn) {
                if let Some(tags) = tag_map.get(path_str) {
                    metadata.tags = tags.clone();
                }
            }
        }
        cache.0.insert(path_str.to_string(), metadata);
        let _ = app.emit("repo-state-changed", ());
    }
}

/// Like `update_repo_cache` but skips the blocking `verify_remote_connectivity` call.
/// Reuses the cached `remote_reachable` value instead. Used by the filesystem watcher
/// so that branch/status changes reflect in the UI immediately without a network round-trip.
pub fn update_repo_cache_local(app: &AppHandle, path_str: &str) {
    let cache = app.state::<RepoCache>();
    let path = Path::new(path_str);

    let git_path = crate::config::load_config(app)
        .map(|c| c.git_path)
        .unwrap_or_else(|_| "git".to_string());

    // Reuse cached remote reachability — avoids blocking network call on every file event
    let cached_remote_reachable = cache.0.get(path_str)
        .map(|entry| entry.remote_reachable)
        .unwrap_or(false);

    if let Some(mut metadata) = get_repo_metadata_local(path, &git_path, cached_remote_reachable) {
        if let Ok(conn) = app.state::<DbPool>().0.lock() {
            if let Ok(tag_map) = batch_fetch_repo_tags(&conn) {
                if let Some(tags) = tag_map.get(path_str) {
                    metadata.tags = tags.clone();
                }
            }
        }
        cache.0.insert(path_str.to_string(), metadata);
        let _ = app.emit("repo-state-changed", ());
    }
}

/// Same as `get_repo_metadata` but accepts a pre-computed `remote_reachable` value
/// instead of making a live network call.
pub fn get_repo_metadata_local(path: &Path, git_path: &str, remote_reachable: bool) -> Option<RepoMetadata> {
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
    // git_path is needed for `git status` — this is a local call, no network
    let sync_status = compute_sync_status(&repo, path, git_path);

    let remote_url = repo.find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|u| u.to_string()));

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
        tags: vec![],
    })
}

pub fn verify_remote_connectivity(path: &str, git_path: &str) -> bool {
    let path = path.to_string();
    let git_path = git_path.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = Command::new(&git_path)
            .args(&["ls-remote", "--exit-code", "--heads", "origin"])
            .current_dir(&path)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let _ = tx.send(result);
    });
    rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap_or(false)
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
    let ext_map: HashMap<&str, &str> = [
        ("rs", "Rust"), ("js", "JavaScript"), ("ts", "TypeScript"),
        ("py", "Python"), ("go", "Go"), ("cpp", "C++"), ("hpp", "C++"), ("cc", "C++"),
        ("c", "C"), ("h", "C"), ("java", "Java"), ("swift", "Swift"),
        ("kt", "Kotlin"), ("php", "PHP"), ("rb", "Ruby"),
        ("html", "HTML"), ("css", "CSS"), ("svelte", "Svelte"),
        ("vue", "Vue"), ("sh", "Shell"), ("sql", "SQL"), ("yaml", "YAML"), ("yml", "YAML"),
        ("cs", "C#"), ("scala", "Scala"), ("dart", "Dart"), ("ex", "Elixir"),
        ("exs", "Elixir"), ("erl", "Erlang"), ("clj", "Clojure"),
    ].iter().cloned().collect();

    let mut counts = HashMap::new();

    let walker = walkdir::WalkDir::new(path)
        .max_depth(3)
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
                if let Some(&lang) = ext_map.get(ext) {
                    *counts.entry(lang.to_string()).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}
