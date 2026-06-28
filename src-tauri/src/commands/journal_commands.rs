use tauri::State;

use crate::domain::journal::{JournalEntry, JournalEntryUpdate, NewJournalEntry};
use crate::error::AppResult;
use crate::repositories::game_repository::SqliteGameRepository;
use crate::repositories::journal_repository::SqliteJournalRepository;
use crate::services::journal_service::JournalService;
use crate::state::AppState;

type Service<'a> = JournalService<'a, SqliteJournalRepository<'a>, SqliteGameRepository<'a>>;

fn with_service<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&Service<'_>) -> AppResult<T>,
) -> AppResult<T> {
    state.db.with_connection(|conn| {
        let journal = SqliteJournalRepository::new(conn);
        let games = SqliteGameRepository::new(conn);
        f(&JournalService::new(&journal, &games))
    })
}

#[tauri::command]
pub fn list_journal_entries(
    state: State<'_, AppState>,
    game_id: i64,
) -> AppResult<Vec<JournalEntry>> {
    with_service(&state, |service| service.list_for_game(game_id))
}

#[tauri::command]
pub fn create_journal_entry(
    state: State<'_, AppState>,
    input: NewJournalEntry,
) -> AppResult<JournalEntry> {
    with_service(&state, |service| service.create(input))
}

#[tauri::command]
pub fn update_journal_entry(
    state: State<'_, AppState>,
    id: i64,
    update: JournalEntryUpdate,
) -> AppResult<JournalEntry> {
    with_service(&state, |service| service.update(id, update))
}

#[tauri::command]
pub fn delete_journal_entry(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    with_service(&state, |service| service.delete(id))
}
