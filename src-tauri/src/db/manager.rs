use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::AppResult;

/// The single gateway to the database. The rest of the application accesses
/// SQLite **only** through this type and must not depend on how the connection is
/// held internally. Today that is a `Mutex<Connection>` (sufficient for a
/// single-user desktop app); it can be replaced with a pool or another strategy
/// without touching services, repositories or commands.
pub struct DatabaseManager {
    connection: Mutex<Connection>,
}

impl DatabaseManager {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection: Mutex::new(connection),
        }
    }

    /// Run `f` with exclusive access to a connection. This is the only way to
    /// obtain a connection, keeping the locking strategy an internal detail.
    pub fn with_connection<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let connection = self.connection.lock().expect("database mutex poisoned");
        f(&connection)
    }

    /// Run `f` with mutable access when an operation targets the database
    /// connection itself (currently profile restore via SQLite's Backup API).
    pub fn with_connection_mut<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> AppResult<T>,
    ) -> AppResult<T> {
        let mut connection = self.connection.lock().expect("database mutex poisoned");
        f(&mut connection)
    }
}
