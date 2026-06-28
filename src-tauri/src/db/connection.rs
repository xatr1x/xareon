use std::path::Path;

use rusqlite::Connection;

use crate::db::migrations;
use crate::error::AppResult;

/// Open (or create) the SQLite database at `path`, enable foreign keys, and run
/// any pending migrations so the connection is ready to use.
pub fn open(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrations::run(&conn)?;
    Ok(conn)
}
