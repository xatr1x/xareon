use crate::domain::statistics::{Statistics, StatsGranularity};
use crate::error::AppResult;
use crate::repositories::statistics_repository::StatisticsRepository;

/// Assembles the Statistics payload. There is no business logic beyond delegating
/// to the repository; gap-filling and labelling are presentation concerns handled
/// by the frontend.
pub struct StatisticsService<'a, R: StatisticsRepository> {
    stats: &'a R,
}

impl<'a, R: StatisticsRepository> StatisticsService<'a, R> {
    pub fn new(stats: &'a R) -> Self {
        Self { stats }
    }

    pub fn statistics(&self, granularity: StatsGranularity) -> AppResult<Statistics> {
        self.stats.statistics(granularity)
    }
}
