use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutableBinding {
    pub id: i64,
    pub game_id: i64,
    pub executable_path: String,
    pub executable_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningProcess {
    pub pid: u32,
    pub executable_path: String,
    pub executable_name: String,
    pub window_title: Option<String>,
    pub has_visible_window: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomaticTrackingState {
    Disabled,
    ProcessNotRunning,
    WaitingForActivity,
    Tracking,
    Afk,
    Suppressed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomaticTrackingStatus {
    pub available: bool,
    pub enabled: bool,
    pub state: AutomaticTrackingState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCapabilities {
    pub automatic_process_tracking: bool,
}

#[derive(Debug, Clone)]
pub struct EnabledBinding {
    pub game_id: i64,
    pub executable_normalized: String,
}
