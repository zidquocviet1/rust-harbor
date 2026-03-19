use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub enum SyncStatus {
    /// Working tree and index are clean, in sync with upstream
    Clean,
    /// Local commits not yet pushed to upstream
    Ahead,
    /// Uncommitted changes (staged, unstaged, or untracked files)
    Dirty,
    /// Upstream has commits not yet pulled
    Behind,
    /// Both local and upstream have diverged commits
    Diverged,
    /// Branch has no remote tracking branch configured, or remote ref not fetched yet
    NoUpstream,
    /// In-progress merge, rebase, or cherry-pick with conflicts
    Conflict,
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
    pub tags: Vec<String>,
}
