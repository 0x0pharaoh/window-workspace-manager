//! Shared error type for the Workset backend.
//!
//! Every fallible backend operation maps to [`WorksetError`]; Tauri command
//! wrappers convert it to `String` via `Display` so the frontend always gets
//! a JSON-friendly error message.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorksetError {
    #[error("database error: {0}")]
    Db(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("windows error: {0}")]
    Windows(String),
    #[error("operation timed out: {0}")]
    Timeout(String),
    #[error("regex error: {0}")]
    Regex(String),
    #[error("json error: {0}")]
    Json(String),
}

impl From<rusqlite::Error> for WorksetError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e.to_string())
    }
}

impl From<std::io::Error> for WorksetError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<serde_json::Error> for WorksetError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e.to_string())
    }
}

impl From<regex::Error> for WorksetError {
    fn from(e: regex::Error) -> Self {
        Self::Regex(e.to_string())
    }
}
