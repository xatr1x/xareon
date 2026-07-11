use rusqlite::Connection;

use crate::domain::statistics::{StatBar, Statistics, StatsGranularity, StatsSummary};
use crate::error::AppResult;

/// How many games to include in the "top games by play time" list.
const TOP_GAMES_LIMIT: i64 = 8;

/// Read-only aggregate queries for the Statistics page. Every figure is derived
/// from **completed** sessions, attributed to the local day they ended on — the
/// same convention as the today/week play-time totals.
pub trait StatisticsRepository {
    fn statistics(&self, granularity: StatsGranularity) -> AppResult<Statistics>;
}

pub struct SqliteStatisticsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteStatisticsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Run a `SELECT key, value` grouping query into a list of bars.
    fn grouped(&self, sql: &str) -> AppResult<Vec<StatBar>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(StatBar { key: row.get(0)?, value: row.get(1)? })
        })?;
        let mut bars = Vec::new();
        for row in rows {
            bars.push(row?);
        }
        Ok(bars)
    }

    fn summary(&self) -> AppResult<StatsSummary> {
        let (total_play_seconds, year_play_seconds) = self.conn.query_row(
            "SELECT \
               COALESCE(SUM(duration_seconds), 0), \
               COALESCE(SUM(CASE WHEN strftime('%Y', ended_at, 'localtime') \
                 = strftime('%Y', 'now', 'localtime') THEN duration_seconds ELSE 0 END), 0) \
             FROM play_sessions WHERE ended_at IS NOT NULL",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )?;

        let (completed_count, playing_count, backlog_count, average_rating) = self.conn.query_row(
            "SELECT \
               SUM(CASE WHEN status IN ('completed', 'completed_100') THEN 1 ELSE 0 END), \
               SUM(CASE WHEN status = 'playing' THEN 1 ELSE 0 END), \
               SUM(CASE WHEN status = 'planned' THEN 1 ELSE 0 END), \
               AVG(rating) \
             FROM games",
            [],
            |row| {
                Ok((
                    row.get::<_, Option<i64>>(0)?.unwrap_or(0),
                    row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                    row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                    row.get::<_, Option<f64>>(3)?,
                ))
            },
        )?;

        Ok(StatsSummary {
            total_play_seconds,
            year_play_seconds,
            completed_count,
            playing_count,
            backlog_count,
            average_rating,
        })
    }

    /// The SQL expression that produces the time-bucket key for `over_time`.
    fn over_time_bucket(granularity: StatsGranularity) -> &'static str {
        match granularity {
            // The Monday that starts the session's local week.
            StatsGranularity::Week => "date(ended_at, 'localtime', 'weekday 0', '-6 days')",
            StatsGranularity::Month => "strftime('%Y-%m', ended_at, 'localtime')",
            StatsGranularity::Year => "strftime('%Y', ended_at, 'localtime')",
        }
    }
}

impl StatisticsRepository for SqliteStatisticsRepository<'_> {
    fn statistics(&self, granularity: StatsGranularity) -> AppResult<Statistics> {
        let summary = self.summary()?;

        let daily = self.grouped(
            "SELECT date(ended_at, 'localtime'), SUM(duration_seconds) \
             FROM play_sessions WHERE ended_at IS NOT NULL \
             GROUP BY 1 ORDER BY 1",
        )?;

        let bucket = Self::over_time_bucket(granularity);
        let over_time = self.grouped(&format!(
            "SELECT {bucket}, SUM(duration_seconds) \
             FROM play_sessions WHERE ended_at IS NOT NULL \
             GROUP BY 1 ORDER BY 1"
        ))?;

        let weekday = self.grouped(
            "SELECT strftime('%w', ended_at, 'localtime'), SUM(duration_seconds) \
             FROM play_sessions WHERE ended_at IS NOT NULL \
             GROUP BY 1",
        )?;

        let top_games = self.grouped(&format!(
            "SELECT g.title, SUM(ps.duration_seconds) AS s \
             FROM play_sessions ps JOIN games g ON g.id = ps.game_id \
             WHERE ps.ended_at IS NOT NULL \
             GROUP BY ps.game_id ORDER BY s DESC LIMIT {TOP_GAMES_LIMIT}"
        ))?;

        // A multi-genre game contributes its play time to each of its genres.
        let genres = self.grouped(
            "SELECT ge.name, SUM(ps.duration_seconds) AS s \
             FROM play_sessions ps \
             JOIN game_genres gg ON gg.game_id = ps.game_id \
             JOIN genres ge ON ge.id = gg.genre_id \
             WHERE ps.ended_at IS NOT NULL \
             GROUP BY ge.id ORDER BY s DESC",
        )?;

        let statuses = self.grouped(
            "SELECT CASE WHEN status = 'completed_100' THEN 'completed' ELSE status END, COUNT(*) \
             FROM games GROUP BY 1",
        )?;

        let ratings = self.grouped(
            "SELECT CAST(rating AS TEXT), COUNT(*) \
             FROM games WHERE rating IS NOT NULL GROUP BY rating ORDER BY rating",
        )?;

        Ok(Statistics {
            summary,
            daily,
            over_time,
            weekday,
            top_games,
            genres,
            statuses,
            ratings,
            granularity,
        })
    }
}
