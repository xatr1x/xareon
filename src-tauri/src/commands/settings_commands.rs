use tauri::{AppHandle, State};

use crate::domain::settings::Settings;
use crate::error::AppResult;
use crate::repositories::settings_repository::SqliteSettingsRepository;
use crate::services::settings_service::SettingsService;
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    state.db.with_connection(|conn| {
        let repo = SqliteSettingsRepository::new(conn);
        SettingsService::new(&repo).get()
    })
}

#[tauri::command]
pub fn update_settings(app: AppHandle, state: State<'_, AppState>, settings: Settings) -> AppResult<Settings> {
    let previous = state.db.with_connection(|conn| {
        let repo = SqliteSettingsRepository::new(conn);
        SettingsService::new(&repo).get()
    })?;
    crate::config::global_shortcut::replace(
        &app,
        previous.play_tracking_shortcut.as_deref(),
        settings.play_tracking_shortcut.as_deref(),
    )?;

    let next_shortcut = settings.play_tracking_shortcut.clone();
    let result = state.db.with_connection(|conn| {
        // Each setting is its own row; a transaction commits them all atomically.
        let tx = conn.unchecked_transaction()?;
        let result = {
            let repo = SqliteSettingsRepository::new(&tx);
            SettingsService::new(&repo).update(settings)?
        };
        tx.commit()?;
        Ok(result)
    });
    if result.is_err() {
        let _ = crate::config::global_shortcut::replace(
            &app,
            next_shortcut.as_deref(),
            previous.play_tracking_shortcut.as_deref(),
        );
    }
    result
}

#[tauri::command]
pub fn suspend_play_tracking_shortcut(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let settings = get_settings(state)?;
    crate::config::global_shortcut::replace(
        &app,
        settings.play_tracking_shortcut.as_deref(),
        None,
    )
}

#[tauri::command]
pub fn resume_play_tracking_shortcut(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let settings = get_settings(state)?;
    crate::config::global_shortcut::replace(
        &app,
        None,
        settings.play_tracking_shortcut.as_deref(),
    )
}
