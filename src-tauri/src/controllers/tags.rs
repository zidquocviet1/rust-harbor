use crate::error::Result;
use crate::services::database::DbPool;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub repo_count: i64,
}

/// Curated colour palette — 12 distinct colours for tags.
pub const TAG_COLORS: &[&str] = &[
    "#6366f1", // Indigo
    "#8b5cf6", // Violet
    "#ec4899", // Pink
    "#f43f5e", // Rose
    "#ef4444", // Red
    "#f97316", // Orange
    "#eab308", // Yellow
    "#22c55e", // Green
    "#14b8a6", // Teal
    "#06b6d4", // Cyan
    "#3b82f6", // Blue
    "#a855f7", // Purple
];

#[tauri::command]
pub async fn list_tags(db: State<'_, DbPool>) -> Result<Vec<Tag>> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.color, COUNT(rt.repo_path) as repo_count
             FROM tags t
             LEFT JOIN repo_tags rt ON rt.tag_id = t.id
             GROUP BY t.id
             ORDER BY t.name COLLATE NOCASE",
        )
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let tags = stmt
        .query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                repo_count: row.get(3)?,
            })
        })
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?
        .collect::<std::result::Result<Vec<Tag>, _>>()
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    Ok(tags)
}

#[tauri::command]
pub async fn create_tag(db: State<'_, DbPool>, name: String, color: String) -> Result<Tag> {
    let trimmed = name.trim().to_string();

    if trimmed.is_empty() {
        return Err(crate::error::Error::DbError("Tag name cannot be empty".to_string()));
    }
    if trimmed.len() > 32 {
        return Err(crate::error::Error::DbError("Tag name must be 32 characters or less".to_string()));
    }

    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?1, ?2)",
        rusqlite::params![trimmed, color],
    )
    .map_err(|e| {
        if let rusqlite::Error::SqliteFailure(ref err, _) = e {
            if err.code == rusqlite::ErrorCode::ConstraintViolation {
                return crate::error::Error::DbError("Tag already exists".to_string());
            }
        }
        crate::error::Error::DbError(e.to_string())
    })?;

    let id = conn.last_insert_rowid();
    Ok(Tag {
        id,
        name: trimmed,
        color,
        repo_count: 0,
    })
}

#[tauri::command]
pub async fn rename_tag(db: State<'_, DbPool>, id: i64, new_name: String) -> Result<()> {
    let trimmed = new_name.trim().to_string();

    if trimmed.is_empty() {
        return Err(crate::error::Error::DbError("Tag name cannot be empty".to_string()));
    }
    if trimmed.len() > 32 {
        return Err(crate::error::Error::DbError("Tag name must be 32 characters or less".to_string()));
    }

    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let affected = conn
        .execute(
            "UPDATE tags SET name = ?1 WHERE id = ?2",
            rusqlite::params![trimmed, id],
        )
        .map_err(|e| {
            if let rusqlite::Error::SqliteFailure(ref err, _) = e {
                if err.code == rusqlite::ErrorCode::ConstraintViolation {
                    return crate::error::Error::DbError("Tag already exists".to_string());
                }
            }
            crate::error::Error::DbError(e.to_string())
        })?;

    if affected == 0 {
        return Err(crate::error::Error::DbError("Tag not found".to_string()));
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_tag(db: State<'_, DbPool>, id: i64) -> Result<()> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let affected = conn
        .execute("DELETE FROM tags WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    if affected == 0 {
        return Err(crate::error::Error::DbError("Tag not found".to_string()));
    }

    Ok(())
}

#[tauri::command]
pub async fn assign_tag(db: State<'_, DbPool>, repo_path: String, tag_id: i64) -> Result<()> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    conn.execute(
        "INSERT OR IGNORE INTO repo_tags (repo_path, tag_id) VALUES (?1, ?2)",
        rusqlite::params![repo_path, tag_id],
    )
    .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn remove_tag(db: State<'_, DbPool>, repo_path: String, tag_id: i64) -> Result<()> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    conn.execute(
        "DELETE FROM repo_tags WHERE repo_path = ?1 AND tag_id = ?2",
        rusqlite::params![repo_path, tag_id],
    )
    .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn get_repo_tags(db: State<'_, DbPool>, repo_path: String) -> Result<Vec<Tag>> {
    let conn = db.0.lock().map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.color, 0 as repo_count
             FROM tags t
             JOIN repo_tags rt ON rt.tag_id = t.id
             WHERE rt.repo_path = ?1
             ORDER BY t.name COLLATE NOCASE",
        )
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    let tags = stmt
        .query_map(rusqlite::params![repo_path], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                repo_count: row.get(3)?,
            })
        })
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?
        .collect::<std::result::Result<Vec<Tag>, _>>()
        .map_err(|e| crate::error::Error::DbError(e.to_string()))?;

    Ok(tags)
}
