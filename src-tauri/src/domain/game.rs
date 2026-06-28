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
/// camelCase to match the TypeScript `Game` interface. `genres` is the list of
/// genre names attached to the game (the normalized many-to-many relation,
/// flattened for the UI).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub genres: Vec<String>,
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
/// (`id`, timestamps) are intentionally absent. `genres` is a list of names; the
/// service normalizes and resolves them to genre entities.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInput {
    pub title: String,
    #[serde(default)]
    pub genres: Vec<String>,
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

/// A flexible query for the game browser: combinable filters plus sorting. All
/// fields are optional/defaulted so the frontend can send only what it needs.
/// This is the single query surface — new filters are added here, not as new
/// commands.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GameQuery {
    /// Case-insensitive substring match on the title.
    pub search: Option<String>,
    /// Match any of these statuses (OR).
    pub statuses: Vec<GameStatus>,
    /// Match any of these platforms (OR).
    pub platforms: Vec<String>,
    /// Genre names to filter by, combined per `genre_match`.
    pub genres: Vec<String>,
    pub genre_match: GenreMatch,
    pub release_year: Option<i64>,
    pub started_year: Option<i64>,
    pub finished_year: Option<i64>,
    /// Games active during this year (started on/before and not finished before).
    pub played_year: Option<i64>,
    pub min_rating: Option<i64>,
    pub max_rating: Option<i64>,
    pub sort: GameSort,
    pub direction: SortDirection,
}

/// How multiple selected genres combine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GenreMatch {
    /// The game has at least one of the genres.
    #[default]
    Any,
    /// The game has all of the genres.
    All,
}

/// Sortable game fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GameSort {
    #[default]
    Title,
    StartedAt,
    FinishedAt,
    ReleaseYear,
    Rating,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}
