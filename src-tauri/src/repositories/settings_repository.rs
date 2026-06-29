use std::collections::HashMap;

use rusqlite::Connection;

use crate::error::AppResult;

/// Key-value persistence for application settings. The repository stays generic
/// over keys so the storage shape never changes as settings are added; mapping
/// keys to typed fields is the service's job.
pub trait SettingsRepository {
    /// All stored settings as a `key → value` map.
    fn get_all(&self) -> AppResult<HashMap<String, String>>;
    /// Insert or overwrite a single setting.
    fn set(&self, key: &str, value: &str) -> AppResult<()>;
}

pub struct SqliteSettingsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteSettingsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl SettingsRepository for SqliteSettingsRepository<'_> {
    fn get_all(&self) -> AppResult<HashMap<String, String>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let settings = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
            .collect::<rusqlite::Result<HashMap<String, String>>>()?;
        Ok(settings)
    }

    fn set(&self, key: &str, value: &str) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }
}
