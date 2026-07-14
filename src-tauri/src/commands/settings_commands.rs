use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::config::device_settings::DeviceSettings;
use crate::domain::settings::Settings;
use crate::error::{AppError, AppResult};
use crate::repositories::settings_repository::SqliteSettingsRepository;
use crate::services::settings_service::SettingsService;
use crate::state::AppState;

fn config_dir(app: &AppHandle) -> AppResult<PathBuf> {
    app.path()
        .app_config_dir()
        .map_err(|error| AppError::Validation(error.to_string()))
}

fn merge_device_settings(mut settings: Settings, device: &DeviceSettings) -> Settings {
    settings.play_tracking_shortcut = device.play_tracking_shortcut.clone();
    settings.play_tracking_shortcut_error = device.shortcut_registration_error.clone();
    settings
}

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, AppState>) -> AppResult<Settings> {
    let profile = state.db.with_connection(|conn| {
        let repo = SqliteSettingsRepository::new(conn);
        SettingsService::new(&repo).get()
    })?;
    let device = DeviceSettings::load(&config_dir(&app)?)?;
    Ok(merge_device_settings(profile, &device))
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<Settings> {
    let config_dir = config_dir(&app)?;
    let previous_device = DeviceSettings::load(&config_dir)?;
    crate::config::global_shortcut::replace(
        &app,
        previous_device.play_tracking_shortcut.as_deref(),
        settings.play_tracking_shortcut.as_deref(),
    )?;

    let next_device = DeviceSettings {
        play_tracking_shortcut: settings.play_tracking_shortcut.clone(),
        shortcut_registration_error: None,
    };
    if let Err(error) = next_device.save(&config_dir) {
        let _ = crate::config::global_shortcut::replace(
            &app,
            next_device.play_tracking_shortcut.as_deref(),
            previous_device.play_tracking_shortcut.as_deref(),
        );
        return Err(error);
    }

    let result = state.db.with_connection(|conn| {
        let tx = conn.unchecked_transaction()?;
        let result = {
            let repo = SqliteSettingsRepository::new(&tx);
            SettingsService::new(&repo).update(settings)?
        };
        tx.commit()?;
        Ok(result)
    });
    match result {
        Ok(profile) => Ok(merge_device_settings(profile, &next_device)),
        Err(error) => {
            let _ = crate::config::global_shortcut::replace(
                &app,
                next_device.play_tracking_shortcut.as_deref(),
                previous_device.play_tracking_shortcut.as_deref(),
            );
            let _ = previous_device.save(&config_dir);
            Err(error)
        }
    }
}

#[tauri::command]
pub fn suspend_play_tracking_shortcut(app: AppHandle) -> AppResult<()> {
    let device = DeviceSettings::load(&config_dir(&app)?)?;
    crate::config::global_shortcut::replace(&app, device.play_tracking_shortcut.as_deref(), None)
}

#[tauri::command]
pub fn resume_play_tracking_shortcut(app: AppHandle) -> AppResult<()> {
    let config_dir = config_dir(&app)?;
    let mut device = DeviceSettings::load(&config_dir)?;
    match crate::config::global_shortcut::replace(
        &app,
        None,
        device.play_tracking_shortcut.as_deref(),
    ) {
        Ok(()) => {
            device.shortcut_registration_error = None;
            device.save(&config_dir)
        }
        Err(error) => {
            device.shortcut_registration_error = Some(error.to_string());
            let _ = device.save(&config_dir);
            Err(error)
        }
    }
}
