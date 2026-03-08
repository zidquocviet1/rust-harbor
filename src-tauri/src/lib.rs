pub mod error;
pub mod config;
pub mod controllers;
pub mod services;

use crate::controllers::{repo, settings};
use crate::services::watcher::RepoWatcher;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tauri::command]
fn is_git_installed() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(RepoWatcher::new())))
        .manage(repo::RepoCache::new())
        .manage(repo::ScanStatus(std::sync::atomic::AtomicBool::new(false)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            is_git_installed,
            repo::list_repos,
            repo::refresh_repos,
            repo::is_scanning,
            repo::get_repo_readme,
            repo::git_fetch,
            repo::git_pull,
            repo::git_push,
            settings::get_config,
            settings::set_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
