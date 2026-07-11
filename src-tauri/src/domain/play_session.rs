use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaySession {
    pub id: i64,
    pub game_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub last_activity_at: String,
    pub duration_seconds: Option<i64>,
}

pub struct ActivePlaySummary {
    pub game_title: String,
    pub elapsed_seconds: i64,
}
