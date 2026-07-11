use tauri::State;

use crate::domain::statistics::{Statistics, StatsGranularity};
use crate::error::AppResult;
use crate::repositories::statistics_repository::SqliteStatisticsRepository;
use crate::services::statistics_service::StatisticsService;
use crate::state::AppState;

fn read<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&StatisticsService<'_, SqliteStatisticsRepository<'_>>) -> AppResult<T>,
) -> AppResult<T> {
    state.db.with_connection(|conn| {
        let repo = SqliteStatisticsRepository::new(conn);
        f(&StatisticsService::new(&repo))
    })
}

#[tauri::command]
pub fn get_statistics(
    state: State<'_, AppState>,
    granularity: StatsGranularity,
) -> AppResult<Statistics> {
    read(&state, |s| s.statistics(granularity))
}
