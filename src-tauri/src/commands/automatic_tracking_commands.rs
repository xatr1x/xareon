use tauri::{AppHandle, Manager, State};

use crate::config::automatic_tracking::AutomaticTrackingRuntime;
use crate::domain::automatic_tracking::{AutomaticTrackingState, AutomaticTrackingStatus, ExecutableBinding, PlatformCapabilities, RunningProcess};
use crate::error::AppResult;
use crate::repositories::game_process_repository::SqliteGameProcessRepository;
use crate::services::automatic_tracking_service::AutomaticTrackingService;
use crate::state::AppState;

#[tauri::command]
pub fn get_platform_capabilities() -> PlatformCapabilities { PlatformCapabilities { automatic_process_tracking: cfg!(target_os = "windows") } }

#[tauri::command]
pub fn list_running_game_processes() -> Vec<RunningProcess> {
    #[cfg(target_os = "windows")]
    { crate::platform::windows::observe_processes().into_iter().map(|o| o.process).collect() }
    #[cfg(not(target_os = "windows"))]
    { Vec::new() }
}

#[tauri::command]
pub fn list_game_executable_bindings(state: State<'_, AppState>, game_id: i64) -> AppResult<Vec<ExecutableBinding>> {
    state.db.with_connection(|conn| AutomaticTrackingService::new(&SqliteGameProcessRepository::new(conn)).bindings(game_id))
}

#[tauri::command]
pub fn add_game_executable_binding(state: State<'_, AppState>, game_id: i64, executable_path: String) -> AppResult<ExecutableBinding> {
    state.db.with_connection(|conn| { let tx = conn.unchecked_transaction()?; let result = AutomaticTrackingService::new(&SqliteGameProcessRepository::new(&tx)).add(game_id, &executable_path)?; tx.commit()?; Ok(result) })
}

#[tauri::command]
pub fn delete_game_executable_binding(state: State<'_, AppState>, game_id: i64, binding_id: i64) -> AppResult<()> {
    state.db.with_connection(|conn| { let tx = conn.unchecked_transaction()?; AutomaticTrackingService::new(&SqliteGameProcessRepository::new(&tx)).delete(game_id, binding_id)?; tx.commit()?; Ok(()) })
}

#[tauri::command]
pub fn set_automatic_tracking_enabled(state: State<'_, AppState>, game_id: i64, enabled: bool) -> AppResult<()> {
    state.db.with_connection(|conn| { let tx = conn.unchecked_transaction()?; AutomaticTrackingService::new(&SqliteGameProcessRepository::new(&tx)).set_enabled(game_id, enabled)?; tx.commit()?; Ok(()) })
}

#[tauri::command]
pub fn get_automatic_tracking_status(app: AppHandle, state: State<'_, AppState>, game_id: i64) -> AppResult<AutomaticTrackingStatus> {
    let enabled = state.db.with_connection(|conn| AutomaticTrackingService::new(&SqliteGameProcessRepository::new(conn)).is_enabled(game_id))?;
    let runtime = app.state::<AutomaticTrackingRuntime>();
    Ok(AutomaticTrackingStatus { available: cfg!(target_os = "windows"), enabled, state: if !enabled { AutomaticTrackingState::Disabled } else { runtime.state(game_id).unwrap_or(AutomaticTrackingState::ProcessNotRunning) } })
}
