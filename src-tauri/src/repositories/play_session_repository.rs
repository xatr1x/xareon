use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::play_session::{
    ActivePlaySummary, DailyPlayTime, PlaySession, PlayTimeTotals, SessionEndReason,
    TrackingSource,
};
use crate::error::{AppError, AppResult};

const WEEK_START_LOCAL: &str = "date('now', 'localtime', 'weekday 0', '-6 days')";

pub trait PlaySessionRepository {
    fn active(&self) -> AppResult<Option<PlaySession>>;
    fn start(&self, game_id: i64) -> AppResult<PlaySession>;
    fn start_automatic(&self, game_id: i64) -> AppResult<PlaySession>;
    fn heartbeat(&self, game_id: i64) -> AppResult<PlaySession>;
    fn stop(&self, game_id: i64) -> AppResult<()>;
    fn stop_with_reason(&self, game_id: i64, reason: SessionEndReason) -> AppResult<()>;
    fn recover_interrupted(&self) -> AppResult<()>;
    fn most_recent_game_id(&self) -> AppResult<Option<i64>>;
    fn active_summary(&self) -> AppResult<Option<ActivePlaySummary>>;
    fn play_time_totals(&self) -> AppResult<PlayTimeTotals>;
    fn game_seconds_today(&self, game_id: i64) -> AppResult<i64>;
    fn list_daily_for_game(&self, game_id: i64, limit: i64) -> AppResult<Vec<DailyPlayTime>>;
}

pub struct SqlitePlaySessionRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqlitePlaySessionRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn map_active(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlaySession> {
        Ok(PlaySession {
            game_id: row.get("game_id")?,
            started_at: row.get("started_at")?,
            last_activity_at: row.get("last_activity_at")?,
            tracking_source: parse_source(&row.get::<_, String>("tracking_source")?),
        })
    }

    fn map_daily(row: &rusqlite::Row<'_>) -> rusqlite::Result<DailyPlayTime> {
        Ok(DailyPlayTime {
            game_id: row.get("game_id")?,
            play_date: row.get("play_date")?,
            duration_seconds: row.get("duration_seconds")?,
            manual_seconds: row.get("manual_seconds")?,
            automatic_seconds: row.get("automatic_seconds")?,
            sessions_count: row.get("sessions_count")?,
            first_started_at: row.get("first_started_at")?,
            last_ended_at: row.get("last_ended_at")?,
        })
    }

    fn finish_at(
        &self,
        session: &PlaySession,
        ended_at: &str,
        _reason: SessionEndReason,
    ) -> AppResult<()> {
        let duration = self.conn.query_row(
            "SELECT MAX(0, CAST(strftime('%s', ?1) AS INTEGER) - CAST(strftime('%s', ?2) AS INTEGER))",
            params![ended_at, session.started_at],
            |row| row.get::<_, i64>(0),
        )?;

        // Convert the UTC interval into local-calendar segments. Each positive
        // segment is upserted into the one daily row for its game and date.
        self.conn.execute(
            "WITH RECURSIVE days(play_date) AS ( \
                 SELECT date(?2, 'localtime') \
                 UNION ALL \
                 SELECT date(play_date, '+1 day') FROM days \
                 WHERE play_date < date(?3, 'localtime') \
             ), segments AS ( \
                 SELECT play_date, \
                    MAX(CAST(strftime('%s', ?2) AS INTEGER), CAST(strftime('%s', play_date, 'utc') AS INTEGER)) AS segment_start, \
                    MIN(CAST(strftime('%s', ?3) AS INTEGER), CAST(strftime('%s', play_date, '+1 day', 'utc') AS INTEGER)) AS segment_end \
                 FROM days \
             ) \
             INSERT INTO daily_play_time ( \
                 game_id, play_date, duration_seconds, manual_seconds, automatic_seconds, \
                 sessions_count, first_started_at, last_ended_at \
             ) \
             SELECT ?1, play_date, segment_end - segment_start, \
                 CASE WHEN ?4 = 'manual' THEN segment_end - segment_start ELSE 0 END, \
                 CASE WHEN ?4 = 'automatic' THEN segment_end - segment_start ELSE 0 END, \
                 1, datetime(segment_start, 'unixepoch'), datetime(segment_end, 'unixepoch') \
             FROM segments WHERE segment_end > segment_start \
             ON CONFLICT(game_id, play_date) DO UPDATE SET \
                 duration_seconds = duration_seconds + excluded.duration_seconds, \
                 manual_seconds = manual_seconds + excluded.manual_seconds, \
                 automatic_seconds = automatic_seconds + excluded.automatic_seconds, \
                 sessions_count = sessions_count + 1, \
                 first_started_at = MIN(first_started_at, excluded.first_started_at), \
                 last_ended_at = MAX(last_ended_at, excluded.last_ended_at), \
                 updated_at = datetime('now')",
            params![session.game_id, session.started_at, ended_at, session.tracking_source.as_str()],
        )?;

        self.conn.execute("DELETE FROM active_play_session WHERE singleton_id = 1", [])?;
        self.conn.execute(
            "UPDATE games SET total_play_time_seconds = total_play_time_seconds + ?1, \
             is_playing_now = 0, last_played_at = ?2, updated_at = datetime('now') WHERE id = ?3",
            params![duration, ended_at, session.game_id],
        )?;
        Ok(())
    }

    fn start_with_source(&self, game_id: i64, source: TrackingSource) -> AppResult<PlaySession> {
        if self.active()?.is_some() {
            return Err(AppError::Validation("another game is already being tracked".into()));
        }
        if !self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM games WHERE id = ?1)",
            [game_id],
            |row| row.get::<_, bool>(0),
        )? {
            return Err(AppError::NotFound);
        }
        self.conn.execute(
            "INSERT INTO active_play_session (singleton_id, game_id, started_at, last_activity_at, tracking_source) \
             VALUES (1, ?1, datetime('now'), datetime('now'), ?2)",
            params![game_id, source.as_str()],
        )?;
        self.conn.execute(
            "UPDATE games SET is_playing_now = 1, updated_at = datetime('now') WHERE id = ?1",
            [game_id],
        )?;
        self.active()?.ok_or_else(|| AppError::Validation("failed to start play session".into()))
    }
}

impl PlaySessionRepository for SqlitePlaySessionRepository<'_> {
    fn active(&self) -> AppResult<Option<PlaySession>> {
        Ok(self.conn.query_row(
            "SELECT game_id, started_at, last_activity_at, tracking_source \
             FROM active_play_session WHERE singleton_id = 1",
            [],
            Self::map_active,
        ).optional()?)
    }

    fn start(&self, game_id: i64) -> AppResult<PlaySession> {
        self.start_with_source(game_id, TrackingSource::Manual)
    }

    fn start_automatic(&self, game_id: i64) -> AppResult<PlaySession> {
        self.start_with_source(game_id, TrackingSource::Automatic)
    }

    fn heartbeat(&self, game_id: i64) -> AppResult<PlaySession> {
        let session = self.active()?.filter(|session| session.game_id == game_id)
            .ok_or_else(|| AppError::Validation("this game has no active session".into()))?;
        self.conn.execute(
            "UPDATE active_play_session SET last_activity_at = datetime('now') \
             WHERE singleton_id = 1 AND game_id = ?1",
            [session.game_id],
        )?;
        self.active()?.ok_or_else(|| AppError::Validation("active session disappeared".into()))
    }

    fn stop(&self, game_id: i64) -> AppResult<()> {
        self.stop_with_reason(game_id, SessionEndReason::Manual)
    }

    fn stop_with_reason(&self, game_id: i64, reason: SessionEndReason) -> AppResult<()> {
        let session = self.active()?.filter(|session| session.game_id == game_id)
            .ok_or_else(|| AppError::Validation("this game has no active session".into()))?;
        let now: String = self.conn.query_row("SELECT datetime('now')", [], |row| row.get(0))?;
        self.finish_at(&session, &now, reason)
    }

    fn recover_interrupted(&self) -> AppResult<()> {
        if let Some(session) = self.active()? {
            let ended_at = session.last_activity_at.clone();
            self.finish_at(&session, &ended_at, SessionEndReason::Recovery)?;
        }
        self.conn.execute(
            "UPDATE games SET is_playing_now = 0 WHERE is_playing_now = 1 \
             AND id NOT IN (SELECT game_id FROM active_play_session)",
            [],
        )?;
        Ok(())
    }

    fn most_recent_game_id(&self) -> AppResult<Option<i64>> {
        Ok(self.conn.query_row(
            "SELECT game_id FROM daily_play_time ORDER BY last_ended_at DESC, game_id DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).optional()?)
    }

    fn active_summary(&self) -> AppResult<Option<ActivePlaySummary>> {
        Ok(self.conn.query_row(
            "SELECT g.title, MAX(0, CAST(strftime('%s', 'now') AS INTEGER) - CAST(strftime('%s', aps.started_at) AS INTEGER)) \
             FROM active_play_session aps JOIN games g ON g.id = aps.game_id \
             WHERE aps.singleton_id = 1",
            [],
            |row| Ok(ActivePlaySummary { game_title: row.get(0)?, elapsed_seconds: row.get(1)? }),
        ).optional()?)
    }

    fn play_time_totals(&self) -> AppResult<PlayTimeTotals> {
        let sql = format!(
            "SELECT COALESCE(SUM(CASE WHEN play_date = date('now', 'localtime') \
                 THEN duration_seconds ELSE 0 END), 0), COALESCE(SUM(duration_seconds), 0) \
             FROM daily_play_time WHERE play_date >= {WEEK_START_LOCAL}"
        );
        Ok(self.conn.query_row(&sql, [], |row| Ok(PlayTimeTotals {
            today_seconds: row.get(0)?,
            week_seconds: row.get(1)?,
        }))?)
    }

    fn game_seconds_today(&self, game_id: i64) -> AppResult<i64> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(SUM(duration_seconds), 0) FROM daily_play_time \
             WHERE game_id = ?1 AND play_date = date('now', 'localtime')",
            [game_id],
            |row| row.get(0),
        )?)
    }

    fn list_daily_for_game(&self, game_id: i64, limit: i64) -> AppResult<Vec<DailyPlayTime>> {
        let mut stmt = self.conn.prepare(
            "SELECT game_id, play_date, duration_seconds, manual_seconds, automatic_seconds, \
             sessions_count, first_started_at, last_ended_at FROM daily_play_time \
             WHERE game_id = ?1 ORDER BY play_date DESC LIMIT ?2",
        )?;
        let days = stmt
            .query_map(params![game_id, limit.clamp(1, 100)], Self::map_daily)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(days)
    }
}

fn parse_source(value: &str) -> TrackingSource {
    match value {
        "automatic" => TrackingSource::Automatic,
        _ => TrackingSource::Manual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE games ( \
                id INTEGER PRIMARY KEY, title TEXT NOT NULL, \
                total_play_time_seconds INTEGER NOT NULL DEFAULT 0, \
                is_playing_now INTEGER NOT NULL DEFAULT 0, last_played_at TEXT, updated_at TEXT \
             ); \
             CREATE TABLE active_play_session ( \
                singleton_id INTEGER PRIMARY KEY CHECK (singleton_id = 1), \
                game_id INTEGER NOT NULL, started_at TEXT NOT NULL, \
                last_activity_at TEXT NOT NULL, tracking_source TEXT NOT NULL \
             ); \
             CREATE TABLE daily_play_time ( \
                game_id INTEGER NOT NULL, play_date TEXT NOT NULL, duration_seconds INTEGER NOT NULL, \
                manual_seconds INTEGER NOT NULL, automatic_seconds INTEGER NOT NULL, \
                sessions_count INTEGER NOT NULL, first_started_at TEXT NOT NULL, \
                last_ended_at TEXT NOT NULL, updated_at TEXT NOT NULL DEFAULT (datetime('now')), \
                PRIMARY KEY (game_id, play_date) \
             ); \
             INSERT INTO games (id, title) VALUES (1, 'Test');",
        ).unwrap();
        conn
    }

    fn local_as_utc(conn: &Connection, value: &str) -> String {
        conn.query_row("SELECT datetime(?1, 'utc')", [value], |row| row.get(0)).unwrap()
    }

    #[test]
    fn repeated_periods_accumulate_in_one_daily_row() {
        let conn = connection();
        let repo = SqlitePlaySessionRepository::new(&conn);
        for (start, end) in [
            ("2026-01-15 10:00:00", "2026-01-15 10:30:00"),
            ("2026-01-15 11:10:00", "2026-01-15 11:15:00"),
            ("2026-01-15 12:30:00", "2026-01-15 14:20:00"),
            ("2026-01-15 16:00:00", "2026-01-15 16:05:00"),
        ] {
            let session = PlaySession {
                game_id: 1,
                started_at: local_as_utc(&conn, start),
                last_activity_at: local_as_utc(&conn, start),
                tracking_source: TrackingSource::Manual,
            };
            repo.finish_at(&session, &local_as_utc(&conn, end), SessionEndReason::Manual).unwrap();
        }

        let day = repo.list_daily_for_game(1, 10).unwrap().pop().unwrap();
        assert_eq!(day.play_date, "2026-01-15");
        assert_eq!(day.duration_seconds, 9_000);
        assert_eq!(day.manual_seconds, 9_000);
        assert_eq!(day.sessions_count, 4);
    }

    #[test]
    fn period_crossing_midnight_is_split_without_losing_seconds() {
        let conn = connection();
        let repo = SqlitePlaySessionRepository::new(&conn);
        let start = local_as_utc(&conn, "2026-01-15 23:50:00");
        let end = local_as_utc(&conn, "2026-01-16 00:20:00");
        let session = PlaySession {
            game_id: 1,
            started_at: start.clone(),
            last_activity_at: start,
            tracking_source: TrackingSource::Automatic,
        };

        repo.finish_at(&session, &end, SessionEndReason::ProcessExit).unwrap();
        let mut days = repo.list_daily_for_game(1, 10).unwrap();
        days.reverse();

        assert_eq!(days.len(), 2);
        assert_eq!((days[0].play_date.as_str(), days[0].duration_seconds), ("2026-01-15", 600));
        assert_eq!((days[1].play_date.as_str(), days[1].duration_seconds), ("2026-01-16", 1_200));
        assert_eq!(days.iter().map(|day| day.duration_seconds).sum::<i64>(), 1_800);
        assert!(days.iter().all(|day| day.automatic_seconds == day.duration_seconds));
    }
}
