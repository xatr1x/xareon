use serde::{Deserialize, Serialize};

/// User-defined personal milestone for a specific game. This intentionally does
/// not encode game-specific achievement types: locations, gear upgrades, endings
/// and custom challenges all fit the same flexible shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AchievementStatus {
    Planned,
    InProgress,
    Completed,
}

impl AchievementStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AchievementStatus::Planned => "planned",
            AchievementStatus::InProgress => "in_progress",
            AchievementStatus::Completed => "completed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "planned" => AchievementStatus::Planned,
            "in_progress" => AchievementStatus::InProgress,
            "completed" => AchievementStatus::Completed,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: i64,
    pub game_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub status: AchievementStatus,
    pub progress_current: Option<i64>,
    pub progress_target: Option<i64>,
    pub progress_unit: Option<String>,
    pub completed_at: Option<String>,
    pub is_hidden: bool,
    pub display_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAchievement {
    pub game_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub status: AchievementStatus,
    pub progress_current: Option<i64>,
    pub progress_target: Option<i64>,
    pub progress_unit: Option<String>,
    pub completed_at: Option<String>,
    pub is_hidden: bool,
    pub display_order: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementUpdate {
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub status: AchievementStatus,
    pub progress_current: Option<i64>,
    pub progress_target: Option<i64>,
    pub progress_unit: Option<String>,
    pub completed_at: Option<String>,
    pub is_hidden: bool,
    pub display_order: i64,
}
