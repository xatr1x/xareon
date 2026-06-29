use crate::domain::settings::Settings;
use crate::error::AppResult;
use crate::repositories::settings_repository::SettingsRepository;

/// Storage keys for the typed settings fields. Keeping them here (the only place
/// that knows both the typed model and the KV store) is the single mapping point
/// when a new setting is added.
const KEY_USER_IDENTIFIER: &str = "user_identifier";
const KEY_GOOGLE_DRIVE_FOLDER: &str = "google_drive_folder";

/// Business rules for application settings: translate between the typed
/// [`Settings`] aggregate and the key-value store.
pub struct SettingsService<'a, R: SettingsRepository> {
    repo: &'a R,
}

impl<'a, R: SettingsRepository> SettingsService<'a, R> {
    pub fn new(repo: &'a R) -> Self {
        Self { repo }
    }

    /// Load all settings. Keys that were never set come back as `None`.
    pub fn get(&self) -> AppResult<Settings> {
        let stored = self.repo.get_all()?;
        let read = |key: &str| stored.get(key).filter(|v| !v.is_empty()).cloned();
        Ok(Settings {
            user_identifier: read(KEY_USER_IDENTIFIER),
            google_drive_folder: read(KEY_GOOGLE_DRIVE_FOLDER),
        })
    }

    /// Persist every setting, then return the freshly stored aggregate. Values are
    /// trimmed; a cleared field is stored as an empty string and reads back as
    /// `None`.
    pub fn update(&self, settings: Settings) -> AppResult<Settings> {
        self.repo
            .set(KEY_USER_IDENTIFIER, &normalize(settings.user_identifier))?;
        self.repo
            .set(KEY_GOOGLE_DRIVE_FOLDER, &normalize(settings.google_drive_folder))?;
        self.get()
    }
}

/// Trim a setting value, treating absence as an empty string.
fn normalize(value: Option<String>) -> String {
    value.map(|v| v.trim().to_string()).unwrap_or_default()
}
