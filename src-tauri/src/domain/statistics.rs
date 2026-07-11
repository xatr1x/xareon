use serde::{Deserialize, Serialize};

/// Bucketing granularity for the "play time over time" series. All statistics
/// are computed over the whole history; this only changes how the time series is
/// grouped for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StatsGranularity {
    Week,
    #[default]
    Month,
    Year,
}

/// Headline figures for the KPI row.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSummary {
    pub total_play_seconds: i64,
    pub year_play_seconds: i64,
    pub completed_count: i64,
    pub playing_count: i64,
    pub backlog_count: i64,
    pub average_rating: Option<f64>,
}

/// One labelled data point. `value` carries seconds for play-time series and a
/// plain count for status/rating series; the `key` is machine-readable (a date,
/// a bucket key, a weekday index, a status, a genre or game name) and the
/// frontend produces the display label.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatBar {
    pub key: String,
    pub value: i64,
}

/// The full statistics payload for the Statistics page, built in one pass.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub summary: StatsSummary,
    /// Seconds per local day, only for days with sessions (drives the heatmap).
    pub daily: Vec<StatBar>,
    /// Seconds per time bucket, grouped by `granularity`.
    pub over_time: Vec<StatBar>,
    /// Seconds per weekday, keyed by SQLite `%w` (0 = Sunday … 6 = Saturday).
    pub weekday: Vec<StatBar>,
    /// Seconds per game, highest first (capped).
    pub top_games: Vec<StatBar>,
    /// Seconds per genre, highest first.
    pub genres: Vec<StatBar>,
    /// Game count per status (`completed_100` folded into `completed`).
    pub statuses: Vec<StatBar>,
    /// Game count per rating value.
    pub ratings: Vec<StatBar>,
    /// The granularity used for `over_time`, echoed back for the UI.
    pub granularity: StatsGranularity,
}
