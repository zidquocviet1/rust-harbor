use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct WorkspaceInsight {
    pub path: String,
    pub repo_count: usize,
    pub last_scan_time: Option<i64>,
    pub scan_status: String, // "Synced", "Warning", "Scanning"
    pub error_details: Option<String>,
}
