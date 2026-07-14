use serde::{Serialize, Serializer};

/// The single error type that flows from repositories up to the Tauri command
/// boundary. It serializes to a plain message string for the frontend.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("file error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid sync metadata: {0}")]
    Json(#[from] serde_json::Error),

    #[error("not found")]
    NotFound,

    #[error("validation error: {0}")]
    Validation(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
