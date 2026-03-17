pub mod models;
pub mod error;
pub mod config;
pub mod controllers;
pub mod services;

use crate::controllers::{repo, settings, tags};
use crate::services::watcher::RepoWatcher;
use crate::services::database;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[tauri::command]
fn is_git_installed(app: tauri::AppHandle) -> bool {
    let git_path = crate::config::load_config(&app)
        .map(|c| c.git_path)
        .unwrap_or_else(|_| "git".to_string());

    std::process::Command::new(git_path)
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
        .setup(|app| {
            // Initialise the SQLite database and register as managed state
            let db_pool = database::init_database(app.handle())
                .expect("Failed to initialise database");
            app.manage(db_pool);
            Ok(())
        })
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
            settings::set_config,
            settings::get_workspace_insights,
            settings::verify_git_path,
            tags::list_tags,
            tags::create_tag,
            tags::rename_tag,
            tags::delete_tag,
            tags::assign_tag,
            tags::remove_tag,
            tags::get_repo_tags,
            repo::get_installed_editors,
            repo::open_in_editor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
