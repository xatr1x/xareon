use std::path::Path;
use std::process::Command;

use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, AppResult};
use crate::repositories::play_session_repository::{
    PlaySessionRepository, SqlitePlaySessionRepository,
};
use crate::state::AppState;
use crate::storage::profile_sync::{BackupManifest, ProfileSyncInfo, ProfileSyncStorage};

fn storage(app: &AppHandle) -> AppResult<ProfileSyncStorage> {
    Ok(ProfileSyncStorage::new(
        app.path()
            .app_config_dir()
            .map_err(|error| AppError::Validation(error.to_string()))?,
        app.path()
            .app_data_dir()
            .map_err(|error| AppError::Validation(error.to_string()))?,
    ))
}

#[tauri::command]
pub fn choose_profile_sync_folder(app: AppHandle) -> AppResult<Option<ProfileSyncInfo>> {
    let Some(folder) = rfd::FileDialog::new()
        .set_title("Choose Google Drive folder")
        .pick_folder()
    else {
        return Ok(None);
    };
    let storage = storage(&app)?;
    storage.set_folder(folder)?;
    let state = app.state::<AppState>();
    state
        .db
        .with_connection(|connection| storage.info(connection))
        .map(Some)
}

#[tauri::command]
pub fn get_profile_sync_info(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<ProfileSyncInfo> {
    let storage = storage(&app)?;
    state
        .db
        .with_connection(|connection| storage.info(connection))
}

#[tauri::command]
pub fn upload_profile_backup(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<BackupManifest> {
    let storage = storage(&app)?;
    state
        .db
        .with_connection(|connection| storage.upload(connection))
}

#[tauri::command]
pub fn restore_profile_backup(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let storage = storage(&app)?;
    state.db.with_connection_mut(|connection| {
        let sessions = SqlitePlaySessionRepository::new(connection);
        if let Some(active) = sessions.active()? {
            sessions.stop(active.game_id)?;
        }
        storage.restore_into(connection)?;
        Ok(())
    })?;
    app.restart()
}

#[tauri::command]
pub fn open_database_folder(app: AppHandle) -> AppResult<()> {
    let path = storage(&app)?.database_path();
    reveal_file(&path)
}

#[cfg(target_os = "macos")]
fn reveal_file(path: &Path) -> AppResult<()> {
    spawn_checked(Command::new("open").arg("-R").arg(path))
}

#[cfg(target_os = "windows")]
fn reveal_file(path: &Path) -> AppResult<()> {
    spawn_checked(Command::new("explorer.exe").arg(format!("/select,{}", path.display())))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn reveal_file(path: &Path) -> AppResult<()> {
    let folder = path
        .parent()
        .ok_or_else(|| AppError::Validation("database path has no parent".into()))?;
    spawn_checked(Command::new("xdg-open").arg(folder))
}

fn spawn_checked(command: &mut Command) -> AppResult<()> {
    command.spawn().map(|_| ()).map_err(AppError::from)
}
