use serde::{Deserialize, Serialize};

/// A single file record within a pull history entry (stored in DB).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullHistoryFile {
    pub id: i64,
    pub pull_id: i64,
    pub file_path: String,
    /// "added" | "modified" | "deleted" | "renamed"
    pub change_type: String,
    pub additions: i64,
    pub deletions: i64,
    pub diff_content: String,
}

/// Lightweight file record — metadata only, no diff_content.
/// Returned by `get_pull_detail` to keep IPC payload small.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullHistoryFileMeta {
    pub id: i64,
    pub pull_id: i64,
    pub file_path: String,
    pub change_type: String,
    pub additions: i64,
    pub deletions: i64,
}

/// Full detail for a pull history entry using lightweight file metadata.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullHistoryDetailMeta {
    pub entry: PullHistoryEntry,
    pub files: Vec<PullHistoryFileMeta>,
}

/// Summary row for the pull history list view.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullHistoryEntry {
    pub id: i64,
    pub repo_path: String,
    pub repo_name: String,
    pub branch: String,
    /// Unix timestamp (seconds) of when the pull was performed.
    pub pulled_at: i64,
    pub commit_before: String,
    pub commit_after: String,
    pub files_changed_count: i64,
    /// Unix timestamp of the before-commit's author date (None for old records).
    pub commit_before_date: Option<i64>,
    /// Unix timestamp of the after-commit's author date (None for old records).
    pub commit_after_date: Option<i64>,
    /// Subject line of the before-commit message.
    pub commit_before_message: Option<String>,
    /// Author name of the before-commit.
    pub commit_before_author: Option<String>,
    /// Subject line of the after-commit message.
    pub commit_after_message: Option<String>,
    /// Author name of the after-commit.
    pub commit_after_author: Option<String>,
}

/// Full detail for a pull history entry — entry + all file records.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullHistoryDetail {
    pub entry: PullHistoryEntry,
    pub files: Vec<PullHistoryFile>,
}

/// Input struct used when inserting a new pull history record.
#[derive(Debug, Clone)]
pub struct NewPullHistoryFile {
    pub file_path: String,
    pub change_type: String,
    pub additions: i64,
    pub deletions: i64,
    pub diff_content: String,
}

/// Input struct used when inserting a new pull history record.
#[derive(Debug, Clone)]
pub struct NewPullHistory {
    pub repo_path: String,
    pub repo_name: String,
    pub branch: String,
    pub pulled_at: i64,
    pub commit_before: String,
    pub commit_after: String,
    pub commit_before_date: Option<i64>,
    pub commit_after_date: Option<i64>,
    pub commit_before_message: Option<String>,
    pub commit_before_author: Option<String>,
    pub commit_after_message: Option<String>,
    pub commit_after_author: Option<String>,
    pub files: Vec<NewPullHistoryFile>,
}

/// Storage size for a single pull history entry.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullEntrySize {
    pub id: i64,
    pub repo_name: String,
    pub branch: String,
    pub pulled_at: i64,
    pub files_changed_count: i64,
    /// Approximate byte count of stored diff content for this entry.
    pub size_bytes: i64,
}

/// Aggregate storage stats for all pull history records.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageStats {
    /// Total bytes across all diff_content rows.
    pub total_bytes: i64,
    /// Per-entry sizes, sorted by size_bytes DESC.
    pub entries: Vec<PullEntrySize>,
}

/// Returned by the `git_pull` Tauri command — extends the old plain String return.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullResult {
    /// Raw stdout from `git pull`.
    pub output: String,
    /// ID of the created pull_history row, or None if no new commits were pulled.
    pub history_id: Option<i64>,
}
