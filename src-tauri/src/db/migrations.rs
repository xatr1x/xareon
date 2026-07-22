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
    Migration {
        name: "0004_achievements",
        sql: include_str!("../migrations/0004_achievements.sql"),
    },
    Migration {
        name: "0005_play_sessions",
        sql: include_str!("../migrations/0005_play_sessions.sql"),
    },
    Migration {
        name: "0006_endless_status",
        sql: include_str!("../migrations/0006_endless_status.sql"),
    },
    Migration {
        name: "0007_automatic_tracking",
        sql: include_str!("../migrations/0007_automatic_tracking.sql"),
    },
    Migration {
        name: "0008_daily_play_time",
        sql: include_str!("../migrations/0008_daily_play_time.sql"),
    },
];

struct Migration {
    #[allow(dead_code)] // kept for logging/diagnostics and to document order
    name: &'static str,
    sql: &'static str,
}

pub fn current_version() -> i64 {
    MIGRATIONS.len() as i64
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_sessions_migrate_to_daily_rows_and_active_singleton() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        for migration in &MIGRATIONS[..7] {
            conn.execute_batch(migration.sql).unwrap();
        }
        conn.execute("INSERT INTO games (id, title) VALUES (1, 'Legacy')", []).unwrap();
        conn.execute_batch(
            "INSERT INTO play_sessions (game_id, started_at, ended_at, last_activity_at, duration_seconds, tracking_source, ended_reason) \
             VALUES (1, datetime('2026-01-15 23:50:00', 'utc'), datetime('2026-01-16 00:20:00', 'utc'), \
                     datetime('2026-01-16 00:20:00', 'utc'), 1800, 'manual', 'manual'); \
             INSERT INTO play_sessions (game_id, started_at, last_activity_at, tracking_source) \
             VALUES (1, datetime('2026-01-16 10:00:00', 'utc'), datetime('2026-01-16 10:05:00', 'utc'), 'automatic');",
        ).unwrap();

        conn.execute_batch(MIGRATIONS[7].sql).unwrap();

        let (rows, seconds, periods): (i64, i64, i64) = conn.query_row(
            "SELECT COUNT(*), SUM(duration_seconds), SUM(sessions_count) FROM daily_play_time",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
        assert_eq!(rows, 2);
        assert_eq!(seconds, 1_800);
        assert_eq!(periods, 2);
        let active: (i64, String) = conn.query_row(
            "SELECT game_id, tracking_source FROM active_play_session WHERE singleton_id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
        assert_eq!(active, (1, "automatic".into()));
    }
}
