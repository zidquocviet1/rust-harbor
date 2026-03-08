use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub enum SyncStatus {
    Clean,
    Ahead,
    Dirty,
    Behind,
    Diverged,
}

#[derive(Debug, Serialize, Clone)]
pub struct RepoMetadata {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub branch: String,
    pub sync_status: SyncStatus,
    pub remote_url: Option<String>,
    pub remote_reachable: bool,
    pub last_modified: i64,
    pub languages: std::collections::HashMap<String, usize>,
}
