use tauri::State;

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
pub fn update_settings(state: State<'_, AppState>, settings: Settings) -> AppResult<Settings> {
    state.db.with_connection(|conn| {
        // Each setting is its own row; a transaction commits them all atomically.
        let tx = conn.unchecked_transaction()?;
        let result = {
            let repo = SqliteSettingsRepository::new(&tx);
            SettingsService::new(&repo).update(settings)?
        };
        tx.commit()?;
        Ok(result)
    })
}
