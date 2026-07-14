use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackingSource { Manual, Automatic }

impl TrackingSource { pub fn as_str(self) -> &'static str { match self { Self::Manual => "manual", Self::Automatic => "automatic" } } }

#[derive(Debug, Clone, Copy)]
pub enum SessionEndReason { Manual, ProcessExit, Afk, Shutdown, Recovery }

impl SessionEndReason { pub fn as_str(self) -> &'static str { match self { Self::Manual => "manual", Self::ProcessExit => "process_exit", Self::Afk => "afk", Self::Shutdown => "shutdown", Self::Recovery => "recovery" } } }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaySession {
    pub id: i64,
    pub game_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub last_activity_at: String,
    pub duration_seconds: Option<i64>,
    pub tracking_source: TrackingSource,
    pub ended_reason: Option<String>,
}

pub struct ActivePlaySummary {
    pub game_title: String,
    pub elapsed_seconds: i64,
}

/// Aggregated play time over recent calendar windows, derived from completed
/// sessions attributed to the local day they ended on. The live contribution of
/// an in-progress session is added by the frontend, which owns the ticking clock.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayTimeTotals {
    pub today_seconds: i64,
    pub week_seconds: i64,
}
