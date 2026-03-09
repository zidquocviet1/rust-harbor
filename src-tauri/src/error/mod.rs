use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum Error {
    GitError(String),
    IoError(String),
    ConfigError(String),
    SystemError(String),
    DbError(String),
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
