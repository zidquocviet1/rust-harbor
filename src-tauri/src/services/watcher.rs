use notify::{Watcher, RecursiveMode, Result as NotifyResult, event::Event};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::AppHandle;

pub struct RepoWatcher {
    watcher: Option<Box<dyn Watcher + Send + Sync>>,
}

impl RepoWatcher {
    pub fn new() -> Self {
        Self { watcher: None }
    }

    pub fn start(&mut self, app: AppHandle, paths: Vec<PathBuf>) -> crate::error::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
            if let Ok(event) = res {
                // Map system events to our needs - we mostly care about file changes in .git
                // For now, let's just send everything to the channel
                let _ = tx.blocking_send(event);
            }
        }).map_err(|e| crate::error::Error::SystemError(format!("Failed to create watcher: {}", e)))?;

        for path in &paths {
            // We watch the .git directory directly for state changes
            let git_path = path.join(".git");
            if git_path.exists() {
                watcher.watch(&git_path, RecursiveMode::Recursive)
                    .map_err(|e| crate::error::Error::SystemError(format!("Failed to watch {:?}: {}", git_path, e)))?;
            }
        }

        self.watcher = Some(Box::new(watcher));

        // Start a listener thread to emit events to Tauri
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Find which repo this path belongs to
                for path in event.paths {
                    for repo_path in &paths {
                        if path.starts_with(repo_path) {
                            if let Some(path_str) = repo_path.to_str() {
                                crate::services::repo_service::update_repo_cache(&app, path_str);
                            }
                            break;
                        }
                    }
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
