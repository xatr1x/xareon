//! Reusable business validation rules shared across services. Keeping these here
//! avoids duplicating checks and keeps validation out of repositories and UI.

use crate::error::{AppError, AppResult};

/// Ensure a required text field is not blank (after trimming).
pub fn require_non_empty(field: &str, value: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{field} must not be empty")));
    }
    Ok(())
}

/// Ensure an optional integer, when present, falls within an inclusive range.
pub fn require_in_range(field: &str, value: Option<i64>, min: i64, max: i64) -> AppResult<()> {
    if let Some(value) = value {
        if value < min || value > max {
            return Err(AppError::Validation(format!(
                "{field} must be between {min} and {max}"
            )));
        }
    }
    Ok(())
}
