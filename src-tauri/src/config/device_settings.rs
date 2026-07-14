use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

const FILE_NAME: &str = "device-settings.json";
pub const DEFAULT_PLAY_TRACKING_SHORTCUT: &str = "CmdOrCtrl+Shift+P";

/// Settings that belong to one installation and must not travel inside a
/// restored profile database. OS shortcuts can differ between Mac and Windows.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSettings {
    pub play_tracking_shortcut: Option<String>,
    pub shortcut_registration_error: Option<String>,
}

impl Default for DeviceSettings {
    fn default() -> Self {
        Self {
            play_tracking_shortcut: Some(DEFAULT_PLAY_TRACKING_SHORTCUT.to_string()),
            shortcut_registration_error: None,
        }
    }
}

impl DeviceSettings {
    pub fn load(config_dir: &Path) -> AppResult<Self> {
        let path = config_dir.join(FILE_NAME);
        if !path.exists() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }

    pub fn save(&self, config_dir: &Path) -> AppResult<()> {
        fs::create_dir_all(config_dir)?;
        let destination = config_dir.join(FILE_NAME);
        let temporary = temporary_path(&destination);
        let mut file = fs::File::create(&temporary)?;
        file.write_all(&serde_json::to_vec_pretty(self)?)?;
        file.sync_all()?;

        if destination.exists() {
            fs::remove_file(&destination)?;
        }
        fs::rename(temporary, destination)?;
        Ok(())
    }
}

fn temporary_path(destination: &Path) -> PathBuf {
    destination.with_extension("json.tmp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_shortcut_round_trips_as_a_device_setting() {
        let directory = tempfile::tempdir().unwrap();
        let settings = DeviceSettings {
            play_tracking_shortcut: None,
            shortcut_registration_error: Some("occupied".into()),
        };
        settings.save(directory.path()).unwrap();

        let restored = DeviceSettings::load(directory.path()).unwrap();
        assert_eq!(restored.play_tracking_shortcut, None);
        assert_eq!(
            restored.shortcut_registration_error.as_deref(),
            Some("occupied")
        );
    }
}
