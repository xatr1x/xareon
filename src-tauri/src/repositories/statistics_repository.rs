use rusqlite::Connection;

use crate::domain::statistics::{StatBar, Statistics, StatsGranularity, StatsSummary};
use crate::error::AppResult;

/// How many games to include in the "top games by play time" list.
const TOP_GAMES_LIMIT: i64 = 8;

/// Read-only aggregate queries for the Statistics page. Every play-time figure
/// is derived from local-calendar daily aggregates.
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
               COALESCE(SUM(CASE WHEN strftime('%Y', play_date) \
                 = strftime('%Y', 'now', 'localtime') THEN duration_seconds ELSE 0 END), 0) \
             FROM daily_play_time",
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
            StatsGranularity::Week => "date(play_date, 'weekday 0', '-6 days')",
            StatsGranularity::Month => "strftime('%Y-%m', play_date)",
            StatsGranularity::Year => "strftime('%Y', play_date)",
        }
    }
}

impl StatisticsRepository for SqliteStatisticsRepository<'_> {
    fn statistics(&self, granularity: StatsGranularity) -> AppResult<Statistics> {
        let summary = self.summary()?;

        let daily = self.grouped(
            "SELECT play_date, SUM(duration_seconds) FROM daily_play_time \
             GROUP BY play_date ORDER BY play_date",
        )?;

        let bucket = Self::over_time_bucket(granularity);
        let over_time = self.grouped(&format!(
            "SELECT {bucket}, SUM(duration_seconds) \
             FROM daily_play_time \
             GROUP BY 1 ORDER BY 1"
        ))?;

        let weekday = self.grouped(
            "SELECT strftime('%w', play_date), SUM(duration_seconds) \
             FROM daily_play_time \
             GROUP BY 1",
        )?;

        let top_games = self.grouped(&format!(
            "SELECT g.title, SUM(dpt.duration_seconds) AS s \
             FROM daily_play_time dpt JOIN games g ON g.id = dpt.game_id \
             GROUP BY dpt.game_id ORDER BY s DESC LIMIT {TOP_GAMES_LIMIT}"
        ))?;

        // A multi-genre game contributes its play time to each of its genres.
        let genres = self.grouped(
            "SELECT ge.name, SUM(dpt.duration_seconds) AS s \
             FROM daily_play_time dpt \
             JOIN game_genres gg ON gg.game_id = dpt.game_id \
             JOIN genres ge ON ge.id = gg.genre_id \
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
