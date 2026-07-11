use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::play_session::PlaySession;
use crate::error::{AppError, AppResult};

pub trait PlaySessionRepository {
    fn active(&self) -> AppResult<Option<PlaySession>>;
    fn start(&self, game_id: i64) -> AppResult<PlaySession>;
    fn heartbeat(&self, game_id: i64) -> AppResult<PlaySession>;
    fn stop(&self, game_id: i64) -> AppResult<()>;
    fn recover_interrupted(&self) -> AppResult<()>;
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
}
