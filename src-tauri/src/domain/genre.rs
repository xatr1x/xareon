use serde::Serialize;

/// A reusable genre entity. `name` keeps the first-seen casing; recognition and
/// deduplication happen on the normalized form (handled by the repository).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Genre {
    pub id: i64,
    pub name: String,
}

/// Normalize a genre name for recognition/deduplication: trimmed and lowercased.
pub fn normalize(name: &str) -> String {
    name.trim().to_lowercase()
}
