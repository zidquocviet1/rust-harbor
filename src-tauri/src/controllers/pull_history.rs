use crate::error::Result;
use crate::models::pull_history::{PullHistoryDetail, PullHistoryEntry, PullHistoryFile};
use crate::services::database::{
    clear_pull_history as db_clear, delete_pull_history_entries, delete_pull_history_entry,
    get_pull_history, get_pull_history_count, get_pull_history_detail, DbPool,
};
use tauri::State;

#[tauri::command]
pub async fn list_pull_history(
    db: State<'_, DbPool>,
    repo_path: Option<String>,
) -> Result<Vec<PullHistoryEntry>> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    get_pull_history(&conn, repo_path.as_deref())
}

#[tauri::command]
pub async fn get_pull_detail(
    db: State<'_, DbPool>,
    pull_id: i64,
) -> Result<PullHistoryDetail> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    let files: Vec<PullHistoryFile> = get_pull_history_detail(&conn, pull_id)?;
    // Fetch the parent entry for the detail response
    let entries = get_pull_history(&conn, None)?;
    let entry = entries
        .into_iter()
        .find(|e| e.id == pull_id)
        .ok_or_else(|| crate::error::Error::DbError(format!("Pull history entry {} not found", pull_id)))?;
    Ok(PullHistoryDetail { entry, files })
}

#[tauri::command]
pub async fn remove_pull_history_entry(db: State<'_, DbPool>, pull_id: i64) -> Result<()> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    delete_pull_history_entry(&conn, pull_id)
}

#[tauri::command]
pub async fn remove_pull_history_entries(
    db: State<'_, DbPool>,
    pull_ids: Vec<i64>,
) -> Result<()> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    delete_pull_history_entries(&conn, &pull_ids)
}

#[tauri::command]
pub async fn clear_all_pull_history(db: State<'_, DbPool>) -> Result<()> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    db_clear(&conn)
}

#[tauri::command]
pub async fn pull_history_count(db: State<'_, DbPool>) -> Result<i64> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    get_pull_history_count(&conn)
}
