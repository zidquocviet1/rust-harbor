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
    pub files: Vec<NewPullHistoryFile>,
}

/// Returned by the `git_pull` Tauri command — extends the old plain String return.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PullResult {
    /// Raw stdout from `git pull`.
    pub output: String,
    /// ID of the created pull_history row, or None if no new commits were pulled.
    pub history_id: Option<i64>,
}
