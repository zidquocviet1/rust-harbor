use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// Wrapper around a SQLite connection managed as Tauri state.
pub struct DbPool(pub Mutex<Connection>);

/// Returns the path to the `harbor.db` file alongside the app config (stable across restarts).
pub fn get_db_path(app_handle: &tauri::AppHandle) -> crate::error::Result<PathBuf> {
    use tauri::Manager;
    let mut path = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| crate::error::Error::SystemError(format!("Failed to get config dir: {}", e)))?;

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

    // Create repositories table for caching
    conn.execute(
        "CREATE TABLE IF NOT EXISTS repositories (
            path             TEXT    PRIMARY KEY,
            name             TEXT    NOT NULL,
            description      TEXT,
            branch           TEXT    NOT NULL,
            sync_status      TEXT    NOT NULL,
            remote_url       TEXT,
            remote_reachable INTEGER NOT NULL,
            last_modified    INTEGER NOT NULL,
            languages        TEXT    NOT NULL -- JSON string
        )",
        [],
    )
    .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    // Create pull_history table — persists permanently, independent of watchlist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pull_history (
            id                    INTEGER PRIMARY KEY AUTOINCREMENT,
            repo_path             TEXT    NOT NULL,
            repo_name             TEXT    NOT NULL,
            branch                TEXT    NOT NULL,
            pulled_at             INTEGER NOT NULL,
            commit_before         TEXT    NOT NULL,
            commit_after          TEXT    NOT NULL,
            files_changed_count   INTEGER NOT NULL DEFAULT 0,
            commit_before_date    INTEGER,
            commit_after_date     INTEGER,
            commit_before_message TEXT,
            commit_before_author  TEXT,
            commit_after_message  TEXT,
            commit_after_author   TEXT
        )",
        [],
    )
    .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    // Migration: add new columns to existing databases (ignored if already present)
    let _ = conn.execute("ALTER TABLE pull_history ADD COLUMN commit_before_date INTEGER", []);
    let _ = conn.execute("ALTER TABLE pull_history ADD COLUMN commit_after_date INTEGER", []);
    let _ = conn.execute("ALTER TABLE pull_history ADD COLUMN commit_before_message TEXT", []);
    let _ = conn.execute("ALTER TABLE pull_history ADD COLUMN commit_before_author TEXT", []);
    let _ = conn.execute("ALTER TABLE pull_history ADD COLUMN commit_after_message TEXT", []);
    let _ = conn.execute("ALTER TABLE pull_history ADD COLUMN commit_after_author TEXT", []);

    // Create pull_history_files table — per-file diff records for each pull event
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pull_history_files (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            pull_id      INTEGER NOT NULL REFERENCES pull_history(id) ON DELETE CASCADE,
            file_path    TEXT    NOT NULL,
            change_type  TEXT    NOT NULL,
            additions    INTEGER NOT NULL DEFAULT 0,
            deletions    INTEGER NOT NULL DEFAULT 0,
            diff_content TEXT    NOT NULL DEFAULT ''
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

/// Saves a batch of repositories to the database, replacing any existing ones with the same path.
pub fn save_repositories(conn: &Connection, repos: &[crate::models::repo::RepoMetadata]) -> crate::error::Result<()> {
    let mut stmt = conn
        .prepare(
            "INSERT OR REPLACE INTO repositories (
                path, name, description, branch, sync_status, 
                remote_url, remote_reachable, last_modified, languages
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    for repo in repos {
        let sync_status = serde_json::to_string(&repo.sync_status).unwrap_or_else(|_| "\"Clean\"".to_string());
        let languages = serde_json::to_string(&repo.languages).unwrap_or_else(|_| "{}".to_string());
        
        stmt.execute(rusqlite::params![
            repo.path,
            repo.name,
            repo.description,
            repo.branch,
            sync_status.trim_matches('"'), // Store as plain string
            repo.remote_url,
            if repo.remote_reachable { 1 } else { 0 },
            repo.last_modified,
            languages
        ])
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    }

    Ok(())
}

/// Loads all cached repositories from the database.
pub fn load_repositories(conn: &Connection) -> crate::error::Result<Vec<crate::models::repo::RepoMetadata>> {
    let mut stmt = conn
        .prepare(
            "SELECT path, name, description, branch, sync_status, 
                    remote_url, remote_reachable, last_modified, languages 
             FROM repositories",
        )
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            let sync_status_str: String = row.get(4)?;
            let languages_str: String = row.get(8)?;
            
            // Map string back to SyncStatus enum
            let sync_status = match sync_status_str.as_str() {
                "Ahead" => crate::models::repo::SyncStatus::Ahead,
                "Dirty" => crate::models::repo::SyncStatus::Dirty,
                "Behind" => crate::models::repo::SyncStatus::Behind,
                "Diverged" => crate::models::repo::SyncStatus::Diverged,
                _ => crate::models::repo::SyncStatus::Clean,
            };

            let languages: std::collections::HashMap<String, usize> = serde_json::from_str(&languages_str)
                .unwrap_or_default();

            Ok(crate::models::repo::RepoMetadata {
                path: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                branch: row.get(3)?,
                sync_status,
                remote_url: row.get(5)?,
                remote_reachable: row.get::<_, i32>(6)? != 0,
                last_modified: row.get(7)?,
                languages,
                tags: vec![],
            })
        })
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let mut repos = Vec::new();
    for row in rows {
        repos.push(row.map_err(|e| crate::error::Error::DbError(e.to_string()))?);
    }

    Ok(repos)
}

/// Clears all repositories from the cache table.
pub fn clear_repositories(conn: &Connection) -> crate::error::Result<()> {
    conn.execute("DELETE FROM repositories", [])
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    Ok(())
}

// ─── Pull History ────────────────────────────────────────────────────────────

/// Inserts a pull history record and all its file records in a single transaction.
pub fn insert_pull_history(
    conn: &Connection,
    entry: &crate::models::pull_history::NewPullHistory,
) -> crate::error::Result<i64> {
    conn.execute(
        "INSERT INTO pull_history (repo_path, repo_name, branch, pulled_at, commit_before, commit_after, files_changed_count, commit_before_date, commit_after_date, commit_before_message, commit_before_author, commit_after_message, commit_after_author)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        rusqlite::params![
            entry.repo_path,
            entry.repo_name,
            entry.branch,
            entry.pulled_at,
            entry.commit_before,
            entry.commit_after,
            entry.files.len() as i64,
            entry.commit_before_date,
            entry.commit_after_date,
            entry.commit_before_message,
            entry.commit_before_author,
            entry.commit_after_message,
            entry.commit_after_author,
        ],
    )
    .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let pull_id = conn.last_insert_rowid();

    for file in &entry.files {
        conn.execute(
            "INSERT INTO pull_history_files (pull_id, file_path, change_type, additions, deletions, diff_content)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                pull_id,
                file.file_path,
                file.change_type,
                file.additions,
                file.deletions,
                file.diff_content,
            ],
        )
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    }

    Ok(pull_id)
}

/// Returns pull history entries ordered by pulled_at DESC, optionally filtered by repo_path.
pub fn get_pull_history(
    conn: &Connection,
    repo_path: Option<&str>,
) -> crate::error::Result<Vec<crate::models::pull_history::PullHistoryEntry>> {
    let (sql, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match repo_path {
        Some(path) => (
            "SELECT id, repo_path, repo_name, branch, pulled_at, commit_before, commit_after, files_changed_count, commit_before_date, commit_after_date, commit_before_message, commit_before_author, commit_after_message, commit_after_author
             FROM pull_history WHERE repo_path = ?1 ORDER BY pulled_at DESC".to_string(),
            vec![Box::new(path.to_string())],
        ),
        None => (
            "SELECT id, repo_path, repo_name, branch, pulled_at, commit_before, commit_after, files_changed_count, commit_before_date, commit_after_date, commit_before_message, commit_before_author, commit_after_message, commit_after_author
             FROM pull_history ORDER BY pulled_at DESC".to_string(),
            vec![],
        ),
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    let params: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    let rows = stmt
        .query_map(params.as_slice(), |row| {
            Ok(crate::models::pull_history::PullHistoryEntry {
                id: row.get(0)?,
                repo_path: row.get(1)?,
                repo_name: row.get(2)?,
                branch: row.get(3)?,
                pulled_at: row.get(4)?,
                commit_before: row.get(5)?,
                commit_after: row.get(6)?,
                files_changed_count: row.get(7)?,
                commit_before_date: row.get(8)?,
                commit_after_date: row.get(9)?,
                commit_before_message: row.get(10)?,
                commit_before_author: row.get(11)?,
                commit_after_message: row.get(12)?,
                commit_after_author: row.get(13)?,
            })
        })
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| crate::error::Error::DbError(e.to_string()))?);
    }
    Ok(entries)
}

/// Returns all file records for a given pull history entry (with diff_content).
pub fn get_pull_history_detail(
    conn: &Connection,
    pull_id: i64,
) -> crate::error::Result<Vec<crate::models::pull_history::PullHistoryFile>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, pull_id, file_path, change_type, additions, deletions, diff_content
             FROM pull_history_files WHERE pull_id = ?1 ORDER BY file_path",
        )
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let rows = stmt
        .query_map(rusqlite::params![pull_id], |row| {
            Ok(crate::models::pull_history::PullHistoryFile {
                id: row.get(0)?,
                pull_id: row.get(1)?,
                file_path: row.get(2)?,
                change_type: row.get(3)?,
                additions: row.get(4)?,
                deletions: row.get(5)?,
                diff_content: row.get(6)?,
            })
        })
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let mut files = Vec::new();
    for row in rows {
        files.push(row.map_err(|e| crate::error::Error::DbError(e.to_string()))?);
    }
    Ok(files)
}

/// Returns file metadata only (no diff_content) for a pull history entry.
/// Much lighter than `get_pull_history_detail` — safe to call for large pulls.
pub fn get_pull_history_files_meta(
    conn: &Connection,
    pull_id: i64,
) -> crate::error::Result<Vec<crate::models::pull_history::PullHistoryFileMeta>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, pull_id, file_path, change_type, additions, deletions
             FROM pull_history_files WHERE pull_id = ?1 ORDER BY file_path",
        )
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let rows = stmt
        .query_map(rusqlite::params![pull_id], |row| {
            Ok(crate::models::pull_history::PullHistoryFileMeta {
                id: row.get(0)?,
                pull_id: row.get(1)?,
                file_path: row.get(2)?,
                change_type: row.get(3)?,
                additions: row.get(4)?,
                deletions: row.get(5)?,
            })
        })
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let mut files = Vec::new();
    for row in rows {
        files.push(row.map_err(|e| crate::error::Error::DbError(e.to_string()))?);
    }
    Ok(files)
}

/// Returns the diff_content for a single file by its ID.
pub fn get_file_diff_content(
    conn: &Connection,
    file_id: i64,
) -> crate::error::Result<String> {
    conn.query_row(
        "SELECT diff_content FROM pull_history_files WHERE id = ?1",
        rusqlite::params![file_id],
        |row| row.get(0),
    )
    .map_err(|e| crate::error::Error::DbError(e.to_string()))
}

/// Returns storage stats: total bytes and per-entry sizes sorted by size DESC.
pub fn get_storage_stats(
    conn: &Connection,
) -> crate::error::Result<crate::models::pull_history::StorageStats> {
    let mut stmt = conn
        .prepare(
            "SELECT ph.id, ph.repo_name, ph.branch, ph.pulled_at, ph.files_changed_count,
                    COALESCE(SUM(LENGTH(CAST(phf.diff_content AS BLOB))), 0) AS size_bytes
             FROM pull_history ph
             LEFT JOIN pull_history_files phf ON phf.pull_id = ph.id
             GROUP BY ph.id
             ORDER BY size_bytes DESC",
        )
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(crate::models::pull_history::PullEntrySize {
                id: row.get(0)?,
                repo_name: row.get(1)?,
                branch: row.get(2)?,
                pulled_at: row.get(3)?,
                files_changed_count: row.get(4)?,
                size_bytes: row.get(5)?,
            })
        })
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row.map_err(|e| crate::error::Error::DbError(e.to_string()))?);
    }

    let total_bytes = entries.iter().map(|e| e.size_bytes).sum();
    Ok(crate::models::pull_history::StorageStats { total_bytes, entries })
}

/// Deletes a single pull history entry and cascades to its file records.
pub fn delete_pull_history_entry(conn: &Connection, pull_id: i64) -> crate::error::Result<()> {
    conn.execute("DELETE FROM pull_history WHERE id = ?1", rusqlite::params![pull_id])
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    Ok(())
}

/// Deletes multiple pull history entries by ID.
pub fn delete_pull_history_entries(conn: &Connection, pull_ids: &[i64]) -> crate::error::Result<()> {
    for id in pull_ids {
        conn.execute("DELETE FROM pull_history WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    }
    Ok(())
}

/// Deletes all pull history records from both tables.
pub fn clear_pull_history(conn: &Connection) -> crate::error::Result<()> {
    conn.execute("DELETE FROM pull_history_files", [])
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    conn.execute("DELETE FROM pull_history", [])
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    Ok(())
}

/// Returns the total count of pull history entries.
pub fn get_pull_history_count(conn: &Connection) -> crate::error::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM pull_history", [], |r| r.get(0))
        .map_err(|e| crate::error::Error::DbError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    fn init_test_schema(conn: &Connection) {
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        conn.execute(
            "CREATE TABLE tags (
                id    INTEGER PRIMARY KEY AUTOINCREMENT,
                name  TEXT NOT NULL UNIQUE COLLATE NOCASE,
                color TEXT NOT NULL DEFAULT '#6366f1'
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE repo_tags (
                repo_path TEXT    NOT NULL,
                tag_id    INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (repo_path, tag_id)
            )",
            [],
        )
        .unwrap();
    }

    #[test]
    fn tag_crud_and_batch_fetch_flow() {
        let conn = Connection::open_in_memory().unwrap();
        init_test_schema(&conn);

        conn.execute("INSERT INTO tags (name, color) VALUES (?1, ?2)", params!["work", "#6366f1"])
            .unwrap();
        conn.execute("INSERT INTO tags (name, color) VALUES (?1, ?2)", params!["personal", "#22c55e"])
            .unwrap();

        let work_id: i64 = conn
            .query_row("SELECT id FROM tags WHERE name = 'work'", [], |r| r.get(0))
            .unwrap();
        let personal_id: i64 = conn
            .query_row("SELECT id FROM tags WHERE name = 'personal'", [], |r| r.get(0))
            .unwrap();

        let repo_path = "/tmp/repo-a";
        conn.execute(
            "INSERT OR IGNORE INTO repo_tags (repo_path, tag_id) VALUES (?1, ?2)",
            params![repo_path, work_id],
        )
        .unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO repo_tags (repo_path, tag_id) VALUES (?1, ?2)",
            params![repo_path, personal_id],
        )
        .unwrap();

        conn.execute(
            "UPDATE tags SET name = ?1 WHERE id = ?2",
            params!["office", work_id],
        )
        .unwrap();

        let map = batch_fetch_repo_tags(&conn).unwrap();
        let tags = map.get(repo_path).cloned().unwrap_or_default();
        assert_eq!(tags, vec!["office".to_string(), "personal".to_string()]);

        conn.execute("DELETE FROM tags WHERE id = ?1", params![personal_id])
            .unwrap();

        let map = batch_fetch_repo_tags(&conn).unwrap();
        let tags = map.get(repo_path).cloned().unwrap_or_default();
        assert_eq!(tags, vec!["office".to_string()]);
    }

    #[test]
    fn cleanup_orphaned_tags_keeps_only_valid_paths() {
        let conn = Connection::open_in_memory().unwrap();
        init_test_schema(&conn);

        conn.execute("INSERT INTO tags (name, color) VALUES (?1, ?2)", params!["work", "#6366f1"])
            .unwrap();
        let tag_id: i64 = conn
            .query_row("SELECT id FROM tags WHERE name = 'work'", [], |r| r.get(0))
            .unwrap();

        conn.execute(
            "INSERT INTO repo_tags (repo_path, tag_id) VALUES (?1, ?2)",
            params!["/repo/keep", tag_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO repo_tags (repo_path, tag_id) VALUES (?1, ?2)",
            params!["/repo/remove", tag_id],
        )
        .unwrap();

        cleanup_orphaned_tags(&conn, &["/repo/keep".to_string()]).unwrap();

        let count_keep: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM repo_tags WHERE repo_path = '/repo/keep'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let count_remove: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM repo_tags WHERE repo_path = '/repo/remove'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(count_keep, 1);
        assert_eq!(count_remove, 0);
    }

    #[test]
    fn sqlite_data_persists_across_connections() {
        let db_path = std::env::temp_dir().join(format!(
            "rust_harbor_test_{}_persistence.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&db_path);

        {
            let conn = Connection::open(&db_path).unwrap();
            init_test_schema(&conn);
            conn.execute("INSERT INTO tags (name, color) VALUES (?1, ?2)", params!["persist", "#3b82f6"])
                .unwrap();
            let tag_id: i64 = conn
                .query_row("SELECT id FROM tags WHERE name = 'persist'", [], |r| r.get(0))
                .unwrap();
            conn.execute(
                "INSERT INTO repo_tags (repo_path, tag_id) VALUES (?1, ?2)",
                params!["/repo/persist", tag_id],
            )
            .unwrap();
        }

        {
            let conn = Connection::open(&db_path).unwrap();
            let map = batch_fetch_repo_tags(&conn).unwrap();
            let tags = map.get("/repo/persist").cloned().unwrap_or_default();
            assert_eq!(tags, vec!["persist".to_string()]);
        }

        let _ = std::fs::remove_file(&db_path);
    }
}
