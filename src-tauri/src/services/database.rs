use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// Wrapper around a SQLite connection managed as Tauri state.
pub struct DbPool(pub Mutex<Connection>);

/// Returns the path to the `harbor.db` file inside the Tauri app data directory.
pub fn get_db_path(app_handle: &tauri::AppHandle) -> crate::error::Result<PathBuf> {
    use tauri::Manager;
    let mut path = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| crate::error::Error::SystemError(format!("Failed to get app data dir: {}", e)))?;

    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    path.push("harbor.db");
    Ok(path)
}

/// Initialises the SQLite database: creates tables if they don't exist and enables WAL mode.
pub fn init_database(app_handle: &tauri::AppHandle) -> crate::error::Result<DbPool> {
    let db_path = get_db_path(app_handle)?;
    let conn = Connection::open(&db_path)
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    // Enable WAL mode for better concurrent read performance and crash resilience
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    // Create tags table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            name  TEXT NOT NULL UNIQUE COLLATE NOCASE,
            color TEXT NOT NULL DEFAULT '#6366f1'
        )",
        [],
    )
    .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    // Create repo_tags join table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS repo_tags (
            repo_path TEXT    NOT NULL,
            tag_id    INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (repo_path, tag_id)
        )",
        [],
    )
    .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    // Ensure foreign keys are enforced
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    Ok(DbPool(Mutex::new(conn)))
}

/// Removes repo_tags entries whose repo_path is not in the given set of valid paths.
pub fn cleanup_orphaned_tags(conn: &Connection, valid_paths: &[String]) -> crate::error::Result<()> {
    if valid_paths.is_empty() {
        // If no valid paths at all, remove everything
        conn.execute("DELETE FROM repo_tags", [])
            .map_err(|e| crate::error::Error::DbError(e.to_string()))?;
        return Ok(());
    }

    let placeholders: Vec<String> = valid_paths.iter().map(|_| "?".to_string()).collect();
    let sql = format!(
        "DELETE FROM repo_tags WHERE repo_path NOT IN ({})",
        placeholders.join(", ")
    );

    let params: Vec<&dyn rusqlite::types::ToSql> = valid_paths
        .iter()
        .map(|p| p as &dyn rusqlite::types::ToSql)
        .collect();

    conn.execute(&sql, params.as_slice())
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    Ok(())
}

/// Returns a mapping of repo_path → Vec<tag name> for all entries in repo_tags.
pub fn batch_fetch_repo_tags(conn: &Connection) -> crate::error::Result<std::collections::HashMap<String, Vec<String>>> {
    let mut stmt = conn
        .prepare(
            "SELECT rt.repo_path, t.name
             FROM repo_tags rt
             JOIN tags t ON t.id = rt.tag_id
             ORDER BY rt.repo_path, t.name",
        )
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for row in rows {
        let (path, tag_name) = row.map_err(|e| crate::error::Error::DbError(e.to_string()))?;
        map.entry(path).or_default().push(tag_name);
    }

    Ok(map)
}
