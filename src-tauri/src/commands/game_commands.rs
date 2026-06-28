use tauri::State;

use crate::domain::game::{Game, GameInput};
use crate::error::AppResult;
use crate::repositories::game_repository::SqliteGameRepository;
use crate::services::game_service::GameService;
use crate::state::AppState;

/// Run a closure with a freshly wired `GameService`, obtaining a connection
/// through the `DatabaseManager` so commands stay unaware of the locking strategy.
fn with_service<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&GameService<'_, SqliteGameRepository<'_>>) -> AppResult<T>,
) -> AppResult<T> {
    state.db.with_connection(|conn| {
        let repo = SqliteGameRepository::new(conn);
        let service = GameService::new(&repo);
        f(&service)
    })
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> AppResult<Vec<Game>> {
    with_service(&state, |service| service.list())
}

#[tauri::command]
pub fn get_game(state: State<'_, AppState>, id: i64) -> AppResult<Game> {
    with_service(&state, |service| service.get(id))
}

#[tauri::command]
pub fn create_game(state: State<'_, AppState>, input: GameInput) -> AppResult<Game> {
    with_service(&state, |service| service.create(input))
}

#[tauri::command]
pub fn update_game(state: State<'_, AppState>, id: i64, input: GameInput) -> AppResult<Game> {
    with_service(&state, |service| service.update(id, input))
}

#[tauri::command]
pub fn delete_game(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    with_service(&state, |service| service.delete(id))
}
