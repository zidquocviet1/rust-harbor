use notify::{Watcher, RecursiveMode, Result as NotifyResult, event::Event};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::AppHandle;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

pub struct RepoWatcher {
    watcher: Option<Box<dyn Watcher + Send + Sync>>,
}

/// Returns false for `.git` sub-paths that carry no meaningful state change:
/// bulk object blobs, reflogs, lock files, and remote-tracking refs.
fn is_relevant_git_path(path: &std::path::Path) -> bool {
    let s = path.to_string_lossy();
    !s.contains("/.git/objects/")
        && !s.contains("/.git/logs/")
        && !s.contains("\\.git\\objects\\")
        && !s.contains("\\.git\\logs\\")
        && !s.ends_with(".lock")
        && !s.ends_with("/FETCH_HEAD")
        && !s.ends_with("\\FETCH_HEAD")
        && !s.ends_with("/ORIG_HEAD")
        && !s.ends_with("\\ORIG_HEAD")
}

impl RepoWatcher {
    pub fn new() -> Self {
        Self { watcher: None }
    }

    pub fn start(&mut self, app: AppHandle, paths: Vec<PathBuf>) -> crate::error::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(200);

        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        }).map_err(|e| crate::error::Error::SystemError(
            format!("Failed to create watcher: {}", e)
        ))?;

        for path in &paths {
            let git_path = path.join(".git");
            if git_path.exists() {
                watcher.watch(&git_path, RecursiveMode::Recursive)
                    .map_err(|e| crate::error::Error::SystemError(
                        format!("Failed to watch {:?}: {}", git_path, e)
                    ))?;
            }
        }

        self.watcher = Some(Box::new(watcher));

        tokio::spawn(async move {
            // repo_path -> time of last relevant event
            let mut pending: HashMap<String, Instant> = HashMap::new();
            let debounce  = Duration::from_millis(300);
            let poll_tick = Duration::from_millis(100);

            loop {
                // Drain events that arrive within the next poll_tick window.
                let deadline = tokio::time::sleep(poll_tick);
                tokio::pin!(deadline);

                loop {
                    tokio::select! {
                        biased;
                        msg = rx.recv() => {
                            match msg {
                                Some(event) => {
                                    for path in &event.paths {
                                        if !is_relevant_git_path(path) {
                                            continue;
                                        }
                                        for repo_path in &paths {
                                            if path.starts_with(repo_path) {
                                                if let Some(path_str) = repo_path.to_str() {
                                                    pending.insert(
                                                        path_str.to_string(),
                                                        Instant::now(),
                                                    );
                                                }
                                                break;
                                            }
                                        }
                                    }
                                }
                                // Channel closed — watcher stopped
                                None => return,
                            }
                        }
                        _ = &mut deadline => break,
                    }
                }

                // Fire update for repos that have been quiet for >= debounce duration.
                let now = Instant::now();
                let ready: Vec<String> = pending
                    .iter()
                    .filter(|(_, t)| now.duration_since(**t) >= debounce)
                    .map(|(k, _)| k.clone())
                    .collect();

                for repo_path in ready {
                    pending.remove(&repo_path);
                    let app_clone = app.clone();
                    let path_clone = repo_path.clone();
                    tokio::task::spawn_blocking(move || {
                        crate::services::repo_service::update_repo_cache_local(
                            &app_clone,
                            &path_clone,
                        );
                    });
                }
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) {
        self.watcher = None;
    }
}

pub type WatcherState = Arc<Mutex<RepoWatcher>>;
