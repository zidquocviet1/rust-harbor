use serde::Serializer;

#[derive(Debug)]
pub enum Error {
    GitError(String),
    IoError(String),
    ConfigError(String),
    SystemError(String),
    DbError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::GitError(e) => write!(f, "Git error: {}", e),
            Error::IoError(e) => write!(f, "IO error: {}", e),
            Error::ConfigError(e) => write!(f, "Config error: {}", e),
            Error::SystemError(e) => write!(f, "{}", e),
            Error::DbError(e) => write!(f, "Database error: {}", e),
        }
    }
}

/// Serialize as a plain string so Tauri IPC returns the message directly
/// (instead of a JSON object like `{"SystemError": "..."}`).
impl serde::Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<git2::Error> for Error {
    fn from(err: git2::Error) -> Self {
        Error::GitError(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::DbError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
