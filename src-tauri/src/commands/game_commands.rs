use tauri::State;

use crate::domain::game::{Game, GameInput, GameQuery};
use crate::error::AppResult;
use crate::repositories::game_repository::SqliteGameRepository;
use crate::repositories::genre_repository::SqliteGenreRepository;
use crate::services::game_service::GameService;
use crate::state::AppState;

type Service<'a> = GameService<'a, SqliteGameRepository<'a>, SqliteGenreRepository<'a>>;

/// Wire a read-only `GameService` over a borrowed connection.
fn read<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&Service<'_>) -> AppResult<T>,
) -> AppResult<T> {
    state.db.with_connection(|conn| {
        let games = SqliteGameRepository::new(conn);
        let genres = SqliteGenreRepository::new(conn);
        f(&GameService::new(&games, &genres))
    })
}

/// Wire a `GameService` inside a transaction so a game and its genres commit
/// atomically.
fn write<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&Service<'_>) -> AppResult<T>,
) -> AppResult<T> {
    state.db.with_connection(|conn| {
        let tx = conn.unchecked_transaction()?;
        let result = {
            let games = SqliteGameRepository::new(&tx);
            let genres = SqliteGenreRepository::new(&tx);
            f(&GameService::new(&games, &genres))?
        };
        tx.commit()?;
        Ok(result)
    })
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>, query: Option<GameQuery>) -> AppResult<Vec<Game>> {
    let query = query.unwrap_or_default();
    read(&state, |service| service.query(&query))
}

#[tauri::command]
pub fn get_game(state: State<'_, AppState>, id: i64) -> AppResult<Game> {
    read(&state, |service| service.get(id))
}

#[tauri::command]
pub fn create_game(state: State<'_, AppState>, input: GameInput) -> AppResult<Game> {
    write(&state, |service| service.create(input))
}

#[tauri::command]
pub fn update_game(state: State<'_, AppState>, id: i64, input: GameInput) -> AppResult<Game> {
    write(&state, |service| service.update(id, input))
}

#[tauri::command]
pub fn delete_game(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    write(&state, |service| service.delete(id))
}
