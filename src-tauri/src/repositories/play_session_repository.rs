use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::play_session::{ActivePlaySummary, PlaySession, PlayTimeTotals};
use crate::error::{AppError, AppResult};

/// Start of the current week (Monday 00:00) as a local calendar date, for use in
/// `date(..., 'localtime') >= ?` comparisons.
const WEEK_START_LOCAL: &str = "date('now', 'localtime', 'weekday 0', '-6 days')";

pub trait PlaySessionRepository {
    fn active(&self) -> AppResult<Option<PlaySession>>;
    fn start(&self, game_id: i64) -> AppResult<PlaySession>;
    fn heartbeat(&self, game_id: i64) -> AppResult<PlaySession>;
    fn stop(&self, game_id: i64) -> AppResult<()>;
    fn recover_interrupted(&self) -> AppResult<()>;
    fn most_recent_game_id(&self) -> AppResult<Option<i64>>;
    fn active_summary(&self) -> AppResult<Option<ActivePlaySummary>>;
    /// Play time from completed sessions across today and the current week.
    fn play_time_totals(&self) -> AppResult<PlayTimeTotals>;
    /// Play time from completed sessions of one game that ended today.
    fn game_seconds_today(&self, game_id: i64) -> AppResult<i64>;
}

pub struct SqlitePlaySessionRepository<'a> { conn: &'a Connection }

impl<'a> SqlitePlaySessionRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self { Self { conn } }

    fn map(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlaySession> {
        Ok(PlaySession {
            id: row.get("id")?, game_id: row.get("game_id")?,
            started_at: row.get("started_at")?, ended_at: row.get("ended_at")?,
            last_activity_at: row.get("last_activity_at")?, duration_seconds: row.get("duration_seconds")?,
        })
    }

    fn finish_at(&self, session: &PlaySession, ended_at: &str) -> AppResult<()> {
        let duration = self.conn.query_row(
            "SELECT MAX(0, CAST(strftime('%s', ?1) AS INTEGER) - CAST(strftime('%s', ?2) AS INTEGER))",
            params![ended_at, session.started_at], |row| row.get::<_, i64>(0),
        )?;
        self.conn.execute(
            "UPDATE play_sessions SET ended_at = ?1, last_activity_at = ?1, duration_seconds = ?2 WHERE id = ?3 AND ended_at IS NULL",
            params![ended_at, duration, session.id],
        )?;
        self.conn.execute(
            "UPDATE games SET total_play_time_seconds = total_play_time_seconds + ?1, is_playing_now = 0, last_played_at = ?2, updated_at = datetime('now') WHERE id = ?3",
            params![duration, ended_at, session.game_id],
        )?;
        Ok(())
    }
}

impl PlaySessionRepository for SqlitePlaySessionRepository<'_> {
    fn active(&self) -> AppResult<Option<PlaySession>> {
        Ok(self.conn.query_row(
            "SELECT id, game_id, started_at, ended_at, last_activity_at, duration_seconds FROM play_sessions WHERE ended_at IS NULL LIMIT 1",
            [], Self::map,
        ).optional()?)
    }

    fn start(&self, game_id: i64) -> AppResult<PlaySession> {
        if self.active()?.is_some() { return Err(AppError::Validation("another game is already being tracked".into())); }
        if !self.conn.query_row("SELECT EXISTS(SELECT 1 FROM games WHERE id = ?1)", [game_id], |r| r.get::<_, bool>(0))? {
            return Err(AppError::NotFound);
        }
        self.conn.execute("INSERT INTO play_sessions (game_id, started_at, last_activity_at) VALUES (?1, datetime('now'), datetime('now'))", [game_id])?;
        self.conn.execute("UPDATE games SET is_playing_now = 1, updated_at = datetime('now') WHERE id = ?1", [game_id])?;
        self.active()?.ok_or_else(|| AppError::Validation("failed to start play session".into()))
    }

    fn heartbeat(&self, game_id: i64) -> AppResult<PlaySession> {
        let session = self.active()?.filter(|s| s.game_id == game_id)
            .ok_or_else(|| AppError::Validation("this game has no active session".into()))?;
        self.conn.execute("UPDATE play_sessions SET last_activity_at = datetime('now') WHERE id = ?1 AND ended_at IS NULL", [session.id])?;
        self.active()?.ok_or_else(|| AppError::Validation("active session disappeared".into()))
    }

    fn stop(&self, game_id: i64) -> AppResult<()> {
        let session = self.active()?.filter(|s| s.game_id == game_id)
            .ok_or_else(|| AppError::Validation("this game has no active session".into()))?;
        let now: String = self.conn.query_row("SELECT datetime('now')", [], |r| r.get(0))?;
        self.finish_at(&session, &now)
    }

    fn recover_interrupted(&self) -> AppResult<()> {
        if let Some(session) = self.active()? {
            let ended_at = session.last_activity_at.clone();
            self.finish_at(&session, &ended_at)?;
        }
        self.conn.execute("UPDATE games SET is_playing_now = 0 WHERE is_playing_now = 1 AND id NOT IN (SELECT game_id FROM play_sessions WHERE ended_at IS NULL)", [])?;
        Ok(())
    }

    fn most_recent_game_id(&self) -> AppResult<Option<i64>> {
        Ok(self.conn.query_row(
            "SELECT game_id FROM play_sessions WHERE ended_at IS NOT NULL ORDER BY ended_at DESC, id DESC LIMIT 1",
            [], |row| row.get(0),
        ).optional()?)
    }

    fn active_summary(&self) -> AppResult<Option<ActivePlaySummary>> {
        Ok(self.conn.query_row(
            "SELECT g.title, MAX(0, CAST(strftime('%s', 'now') AS INTEGER) - CAST(strftime('%s', ps.started_at) AS INTEGER)) \
             FROM play_sessions ps JOIN games g ON g.id = ps.game_id WHERE ps.ended_at IS NULL",
            [],
            |row| Ok(ActivePlaySummary {
                game_title: row.get(0)?,
                elapsed_seconds: row.get(1)?,
            }),
        ).optional()?)
    }

    fn play_time_totals(&self) -> AppResult<PlayTimeTotals> {
        // A single scan over this week's completed sessions; today is a subset of
        // the week, so both totals come from the same narrow row set.
        let sql = format!(
            "SELECT \
               COALESCE(SUM(CASE WHEN date(ended_at, 'localtime') = date('now', 'localtime') \
                 THEN duration_seconds ELSE 0 END), 0), \
               COALESCE(SUM(duration_seconds), 0) \
             FROM play_sessions \
             WHERE ended_at IS NOT NULL AND date(ended_at, 'localtime') >= {WEEK_START_LOCAL}"
        );
        Ok(self.conn.query_row(&sql, [], |row| {
            Ok(PlayTimeTotals { today_seconds: row.get(0)?, week_seconds: row.get(1)? })
        })?)
    }

    fn game_seconds_today(&self, game_id: i64) -> AppResult<i64> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(SUM(duration_seconds), 0) FROM play_sessions \
             WHERE game_id = ?1 AND ended_at IS NOT NULL \
               AND date(ended_at, 'localtime') = date('now', 'localtime')",
            [game_id],
            |row| row.get(0),
        )?)
    }
}
