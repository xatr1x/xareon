use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::automatic_tracking::{EnabledBinding, ExecutableBinding};
use crate::error::{AppError, AppResult};

pub trait GameProcessRepository {
    fn list_for_game(&self, game_id: i64) -> AppResult<Vec<ExecutableBinding>>;
    fn add(&self, game_id: i64, executable_path: &str) -> AppResult<ExecutableBinding>;
    fn delete(&self, game_id: i64, binding_id: i64) -> AppResult<()>;
    fn set_enabled(&self, game_id: i64, enabled: bool) -> AppResult<()>;
    fn is_enabled(&self, game_id: i64) -> AppResult<bool>;
    fn enabled_bindings(&self) -> AppResult<Vec<EnabledBinding>>;
}

pub struct SqliteGameProcessRepository<'a> { conn: &'a Connection }

impl<'a> SqliteGameProcessRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self { Self { conn } }

    fn normalized(path: &str) -> String {
        path.trim().replace('/', "\\").to_lowercase()
    }

    fn map(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExecutableBinding> {
        Ok(ExecutableBinding {
            id: row.get("id")?, game_id: row.get("game_id")?,
            executable_path: row.get("executable_path")?,
            executable_name: row.get("executable_name")?, created_at: row.get("created_at")?,
        })
    }
}

impl GameProcessRepository for SqliteGameProcessRepository<'_> {
    fn list_for_game(&self, game_id: i64) -> AppResult<Vec<ExecutableBinding>> {
        let mut stmt = self.conn.prepare("SELECT id, game_id, executable_path, executable_name, created_at FROM game_executable_bindings WHERE game_id = ?1 ORDER BY executable_name, id")?;
        let rows = stmt.query_map([game_id], Self::map)?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn add(&self, game_id: i64, executable_path: &str) -> AppResult<ExecutableBinding> {
        if !self.conn.query_row("SELECT EXISTS(SELECT 1 FROM games WHERE id = ?1)", [game_id], |r| r.get::<_, bool>(0))? { return Err(AppError::NotFound); }
        let path = executable_path.trim();
        if path.is_empty() { return Err(AppError::Validation("executable path is required".into())); }
        let name = Path::new(path).file_name().and_then(|v| v.to_str()).filter(|v| !v.is_empty())
            .ok_or_else(|| AppError::Validation("invalid executable path".into()))?;
        if !name.to_lowercase().ends_with(".exe") { return Err(AppError::Validation("the selected process is not an executable".into())); }
        let normalized = Self::normalized(path);
        let existing: Option<i64> = self.conn.query_row("SELECT game_id FROM game_executable_bindings WHERE executable_normalized = ?1", [&normalized], |r| r.get(0)).optional()?;
        if let Some(owner) = existing {
            let message = if owner == game_id { "this executable is already linked to the game" } else { "this executable is linked to another game" };
            return Err(AppError::Validation(message.into()));
        }
        self.conn.execute("INSERT INTO game_executable_bindings (game_id, executable_path, executable_normalized, executable_name) VALUES (?1, ?2, ?3, ?4)", params![game_id, path, normalized, name])?;
        let id = self.conn.last_insert_rowid();
        Ok(self.conn.query_row("SELECT id, game_id, executable_path, executable_name, created_at FROM game_executable_bindings WHERE id = ?1", [id], Self::map)?)
    }

    fn delete(&self, game_id: i64, binding_id: i64) -> AppResult<()> {
        if self.conn.execute("DELETE FROM game_executable_bindings WHERE id = ?1 AND game_id = ?2", params![binding_id, game_id])? == 0 { return Err(AppError::NotFound); }
        Ok(())
    }

    fn set_enabled(&self, game_id: i64, enabled: bool) -> AppResult<()> {
        if enabled && self.list_for_game(game_id)?.is_empty() { return Err(AppError::Validation("link at least one executable before enabling automatic tracking".into())); }
        if self.conn.execute("UPDATE games SET automatic_tracking_enabled = ?1, updated_at = datetime('now') WHERE id = ?2", params![enabled, game_id])? == 0 { return Err(AppError::NotFound); }
        Ok(())
    }

    fn is_enabled(&self, game_id: i64) -> AppResult<bool> {
        Ok(self.conn.query_row("SELECT automatic_tracking_enabled FROM games WHERE id = ?1", [game_id], |r| r.get(0)).optional()?.ok_or(AppError::NotFound)?)
    }

    fn enabled_bindings(&self) -> AppResult<Vec<EnabledBinding>> {
        let mut stmt = self.conn.prepare("SELECT b.game_id, b.executable_normalized FROM game_executable_bindings b JOIN games g ON g.id = b.game_id WHERE g.automatic_tracking_enabled = 1")?;
        let rows = stmt.query_map([], |row| Ok(EnabledBinding { game_id: row.get(0)?, executable_normalized: row.get(1)? }))?.collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}
