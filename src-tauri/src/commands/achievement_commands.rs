use tauri::State;

use crate::domain::achievement::{Achievement, AchievementUpdate, NewAchievement};
use crate::error::AppResult;
use crate::repositories::achievement_repository::SqliteAchievementRepository;
use crate::repositories::game_repository::SqliteGameRepository;
use crate::services::achievement_service::AchievementService;
use crate::state::AppState;

type Service<'a> =
    AchievementService<'a, SqliteAchievementRepository<'a>, SqliteGameRepository<'a>>;

fn with_service<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&Service<'_>) -> AppResult<T>,
) -> AppResult<T> {
    state.db.with_connection(|conn| {
        let achievements = SqliteAchievementRepository::new(conn);
        let games = SqliteGameRepository::new(conn);
        f(&AchievementService::new(&achievements, &games))
    })
}

#[tauri::command]
pub fn list_achievements(
    state: State<'_, AppState>,
    game_id: i64,
) -> AppResult<Vec<Achievement>> {
    with_service(&state, |service| service.list_for_game(game_id))
}

#[tauri::command]
pub fn create_achievement(
    state: State<'_, AppState>,
    input: NewAchievement,
) -> AppResult<Achievement> {
    with_service(&state, |service| service.create(input))
}

#[tauri::command]
pub fn update_achievement(
    state: State<'_, AppState>,
    id: i64,
    update: AchievementUpdate,
) -> AppResult<Achievement> {
    with_service(&state, |service| service.update(id, update))
}

#[tauri::command]
pub fn set_achievement_progress(
    state: State<'_, AppState>,
    id: i64,
    progress_current: i64,
) -> AppResult<Achievement> {
    with_service(&state, |service| service.set_progress(id, progress_current))
}

#[tauri::command]
pub fn complete_achievement(state: State<'_, AppState>, id: i64) -> AppResult<Achievement> {
    with_service(&state, |service| service.complete(id))
}

#[tauri::command]
pub fn reopen_achievement(state: State<'_, AppState>, id: i64) -> AppResult<Achievement> {
    with_service(&state, |service| service.reopen(id))
}

#[tauri::command]
pub fn delete_achievement(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    with_service(&state, |service| service.delete(id))
}
