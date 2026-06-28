use serde::{Deserialize, Serialize};

/// Where a game sits in the player's journey. The wire/storage representation is
/// the snake_case string (e.g. `completed_100`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    Planned,
    Playing,
    Paused,
    Completed,
    #[serde(rename = "completed_100")]
    Completed100,
    Dropped,
}

impl GameStatus {
    /// Canonical string used both on the wire and in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            GameStatus::Planned => "planned",
            GameStatus::Playing => "playing",
            GameStatus::Paused => "paused",
            GameStatus::Completed => "completed",
            GameStatus::Completed100 => "completed_100",
            GameStatus::Dropped => "dropped",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "planned" => GameStatus::Planned,
            "playing" => GameStatus::Playing,
            "paused" => GameStatus::Paused,
            "completed" => GameStatus::Completed,
            "completed_100" => GameStatus::Completed100,
            "dropped" => GameStatus::Dropped,
            _ => return None,
        })
    }
}

/// A game as stored and returned to the UI. Field names cross the boundary in
/// camelCase to match the TypeScript `Game` interface.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub genre: Option<String>,
    pub platform: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_year: Option<i64>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub status: GameStatus,
    pub rating: Option<i64>,
    pub cover_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// User-supplied fields for creating or updating a game. Server-managed columns
/// (`id`, timestamps) are intentionally absent.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInput {
    pub title: String,
    pub genre: Option<String>,
    pub platform: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_year: Option<i64>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub status: GameStatus,
    pub rating: Option<i64>,
    pub cover_path: Option<String>,
}
