use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct EditorInfo {
    pub id: String,
    pub name: String,
    pub icon: String, // icon identifier or path
}
