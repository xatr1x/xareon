use rusqlite::Connection;

use crate::error::AppResult;

/// Ordered list of migrations. Each entry is embedded at compile time. To add a
/// migration, append a new `.sql` file and a line here — never edit an applied one.
const MIGRATIONS: &[Migration] = &[
    Migration {
        name: "0001_init",
        sql: include_str!("../migrations/0001_init.sql"),
    },
    Migration {
        name: "0002_genres_journal",
        sql: include_str!("../migrations/0002_genres_journal.sql"),
    },
    Migration {
        name: "0003_settings",
        sql: include_str!("../migrations/0003_settings.sql"),
    },
];

struct Migration {
    #[allow(dead_code)] // kept for logging/diagnostics and to document order
    name: &'static str,
    sql: &'static str,
}

/// Apply every migration whose 1-based index is greater than the database's
/// current `user_version`, advancing `user_version` as it goes.
pub fn run(conn: &Connection) -> AppResult<()> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    for (index, migration) in MIGRATIONS.iter().enumerate() {
        let version = (index + 1) as i64;
        if version > current {
            conn.execute_batch(migration.sql)?;
            // PRAGMA user_version doesn't accept bound parameters.
            conn.execute_batch(&format!("PRAGMA user_version = {version};"))?;
        }
    }

    Ok(())
}
