use serde::{Deserialize, Serialize};

/// The application's settings, read and saved as a single aggregate. Each field
/// is optional so the UI can present blanks until the user fills them in.
///
/// This is the centralized settings model shared by future features. Adding a
/// setting is a local change: add a field here, then map its storage key in
/// `SettingsService`. No schema migration is needed (settings are stored as
/// key-value rows).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Human-readable, user-chosen public identifier (not a UUID). Later used as
    /// the user's public name, their Google Drive folder name, and the handle
    /// shared with friends.
    pub user_identifier: Option<String>,
    /// URL of the user's shared Google Drive folder. Stored now for the future
    /// synchronization system; the integration itself is out of scope here.
    pub google_drive_folder: Option<String>,
    /// Global Play/Stop toggle accelerator. `None` disables the shortcut.
    pub play_tracking_shortcut: Option<String>,
}
