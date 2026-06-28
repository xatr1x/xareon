use tauri::State;

use crate::domain::genre::Genre;
use crate::error::AppResult;
use crate::repositories::genre_repository::SqliteGenreRepository;
use crate::services::genre_service::GenreService;
use crate::state::AppState;

#[tauri::command]
pub fn list_genres(state: State<'_, AppState>) -> AppResult<Vec<Genre>> {
    state.db.with_connection(|conn| {
        let repo = SqliteGenreRepository::new(conn);
        GenreService::new(&repo).list()
    })
}
