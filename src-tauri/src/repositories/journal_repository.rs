use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::journal::{JournalEntry, NewJournalEntry};
use crate::error::{AppError, AppResult};

/// Persistence operations for journal entries.
pub trait JournalRepository {
    /// Entries for a game, newest first.
    fn list_for_game(&self, game_id: i64) -> AppResult<Vec<JournalEntry>>;
    fn get(&self, id: i64) -> AppResult<JournalEntry>;
    fn create(&self, entry: &NewJournalEntry) -> AppResult<i64>;
    fn update_body(&self, id: i64, body: &str) -> AppResult<()>;
    fn delete(&self, id: i64) -> AppResult<()>;
}

const COLUMNS: &str = "id, game_id, body, created_at, updated_at";

pub struct SqliteJournalRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteJournalRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn map_row(row: &Row<'_>) -> rusqlite::Result<JournalEntry> {
        Ok(JournalEntry {
            id: row.get("id")?,
            game_id: row.get("game_id")?,
            body: row.get("body")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

impl JournalRepository for SqliteJournalRepository<'_> {
    fn list_for_game(&self, game_id: i64) -> AppResult<Vec<JournalEntry>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM journal_entries WHERE game_id = ?1 \
             ORDER BY created_at DESC, id DESC"
        ))?;
        let entries = stmt
            .query_map([game_id], Self::map_row)?
            .collect::<rusqlite::Result<Vec<JournalEntry>>>()?;
        Ok(entries)
    }

    fn get(&self, id: i64) -> AppResult<JournalEntry> {
        self.conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM journal_entries WHERE id = ?1"),
                [id],
                Self::map_row,
            )
            .optional()?
            .ok_or(AppError::NotFound)
    }

    fn create(&self, entry: &NewJournalEntry) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO journal_entries (game_id, body) VALUES (?1, ?2)",
            rusqlite::params![entry.game_id, entry.body],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    fn update_body(&self, id: i64, body: &str) -> AppResult<()> {
        let affected = self.conn.execute(
            "UPDATE journal_entries SET body = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![body, id],
        )?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    fn delete(&self, id: i64) -> AppResult<()> {
        let affected = self
            .conn
            .execute("DELETE FROM journal_entries WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}
