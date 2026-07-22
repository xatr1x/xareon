use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackingSource { Manual, Automatic }

impl TrackingSource { pub fn as_str(self) -> &'static str { match self { Self::Manual => "manual", Self::Automatic => "automatic" } } }

#[derive(Debug, Clone, Copy)]
pub enum SessionEndReason { Manual, ProcessExit, Afk, Shutdown, Recovery }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaySession {
    pub game_id: i64,
    pub started_at: String,
    pub last_activity_at: String,
    pub tracking_source: TrackingSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyPlayTime {
    pub game_id: i64,
    pub play_date: String,
    pub duration_seconds: i64,
    pub manual_seconds: i64,
    pub automatic_seconds: i64,
    pub sessions_count: i64,
    pub first_started_at: String,
    pub last_ended_at: String,
}

pub struct ActivePlaySummary {
    pub game_title: String,
    pub elapsed_seconds: i64,
}

/// Aggregated play time over recent local-calendar windows. Completed periods
/// come from daily aggregates; the frontend adds the ticking active contribution.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayTimeTotals {
    pub today_seconds: i64,
    pub week_seconds: i64,
}
