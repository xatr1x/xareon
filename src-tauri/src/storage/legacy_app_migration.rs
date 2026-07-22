use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags};

use crate::error::{AppError, AppResult};

const LEGACY_IDENTIFIER: &str = "com.xareon.app";
const LEGACY_DATABASE: &str = "xareon.db";
const DATABASE: &str = "xavendrix.db";
const COMPANION_FILES: &[&str] = &["device-settings.json", "profile-sync.json"];

/// Move the pre-rename profile into the Xavendrix namespace. Migration is
/// deliberately conservative: an existing profile is replaced only when its
/// database contains no games and no profile settings.
pub fn migrate_if_needed(data_dir: &Path, config_dir: &Path) -> AppResult<bool> {
    let Some(legacy_data_dir) = sibling_namespace(data_dir) else { return Ok(false) };
    let legacy_database = legacy_data_dir.join(LEGACY_DATABASE);
    if !legacy_database.is_file() || !profile_is_empty(&data_dir.join(DATABASE))? {
        return Ok(false);
    }
    validate_database(&legacy_database)?;

    fs::create_dir_all(data_dir)?;
    copy_directory_if_present(&legacy_data_dir.join("backups"), &data_dir.join("backups"))?;
    copy_companions(&legacy_data_dir, data_dir)?;

    if config_dir != data_dir {
        if let Some(legacy_config_dir) = sibling_namespace(config_dir) {
            copy_companions(&legacy_config_dir, config_dir)?;
        }
    }

    replace_database(&legacy_database, &data_dir.join(DATABASE), data_dir)?;
    Ok(true)
}

fn sibling_namespace(current: &Path) -> Option<PathBuf> {
    current.parent().map(|parent| parent.join(LEGACY_IDENTIFIER))
}

fn profile_is_empty(database: &Path) -> AppResult<bool> {
    if !database.exists() {
        return Ok(true);
    }
    let connection = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let has_games_table: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'games')",
        [],
        |row| row.get(0),
    )?;
    if !has_games_table {
        return Ok(false);
    }
    let games: i64 = connection.query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))?;
    let settings: i64 = connection.query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))?;
    Ok(games == 0 && settings == 0)
}

fn validate_database(database: &Path) -> AppResult<()> {
    let connection = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(AppError::Validation(format!(
            "legacy profile database failed integrity check: {integrity}"
        )));
    }
    Ok(())
}

fn replace_database(source: &Path, destination: &Path, data_dir: &Path) -> AppResult<()> {
    let temporary = data_dir.join(".xavendrix-legacy-import.tmp");
    if temporary.exists() {
        fs::remove_file(&temporary)?;
    }
    fs::copy(source, &temporary)?;
    validate_database(&temporary)?;

    let previous = if destination.exists() {
        let backups = data_dir.join("backups");
        fs::create_dir_all(&backups)?;
        let path = backups.join(format!("pre-legacy-import-{}.sqlite", timestamp()?));
        fs::rename(destination, &path)?;
        Some(path)
    } else {
        None
    };

    if let Err(error) = fs::rename(&temporary, destination) {
        if let Some(previous) = previous {
            let _ = fs::rename(previous, destination);
        }
        return Err(error.into());
    }
    Ok(())
}

fn copy_companions(source_dir: &Path, destination_dir: &Path) -> AppResult<()> {
    fs::create_dir_all(destination_dir)?;
    for name in COMPANION_FILES {
        let source = source_dir.join(name);
        if source.is_file() {
            fs::copy(source, destination_dir.join(name))?;
        }
    }
    Ok(())
}

fn copy_directory_if_present(source: &Path, destination: &Path) -> AppResult<()> {
    if !source.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory_if_present(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn timestamp() -> AppResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| AppError::Validation("system clock is before Unix epoch".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database(path: &Path, title: Option<&str>, setting: Option<&str>) {
        let connection = Connection::open(path).unwrap();
        connection.execute_batch(
            "CREATE TABLE games (id INTEGER PRIMARY KEY, title TEXT NOT NULL); \
             CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        ).unwrap();
        if let Some(title) = title {
            connection.execute("INSERT INTO games (title) VALUES (?1)", [title]).unwrap();
        }
        if let Some(value) = setting {
            connection.execute("INSERT INTO settings VALUES ('user_identifier', ?1)", [value]).unwrap();
        }
    }

    #[test]
    fn migrates_legacy_profile_over_an_empty_new_profile() {
        let root = tempfile::tempdir().unwrap();
        let legacy = root.path().join(LEGACY_IDENTIFIER);
        let current = root.path().join("com.xavendrix.app");
        fs::create_dir_all(&legacy).unwrap();
        fs::create_dir_all(&current).unwrap();
        database(&legacy.join(LEGACY_DATABASE), Some("Migrated game"), Some("vitalii"));
        database(&current.join(DATABASE), None, None);
        fs::write(legacy.join("profile-sync.json"), "legacy-sync").unwrap();

        assert!(migrate_if_needed(&current, &current).unwrap());
        let migrated = Connection::open(current.join(DATABASE)).unwrap();
        assert_eq!(migrated.query_row("SELECT title FROM games", [], |row| row.get::<_, String>(0)).unwrap(), "Migrated game");
        assert_eq!(fs::read_to_string(current.join("profile-sync.json")).unwrap(), "legacy-sync");
        assert!(current.join("backups").read_dir().unwrap().any(|entry| entry.unwrap().file_name().to_string_lossy().starts_with("pre-legacy-import-")));
    }

    #[test]
    fn never_overwrites_a_non_empty_new_profile() {
        let root = tempfile::tempdir().unwrap();
        let legacy = root.path().join(LEGACY_IDENTIFIER);
        let current = root.path().join("com.xavendrix.app");
        fs::create_dir_all(&legacy).unwrap();
        fs::create_dir_all(&current).unwrap();
        database(&legacy.join(LEGACY_DATABASE), Some("Legacy"), None);
        database(&current.join(DATABASE), Some("Current"), None);

        assert!(!migrate_if_needed(&current, &current).unwrap());
        let current_database = Connection::open(current.join(DATABASE)).unwrap();
        assert_eq!(current_database.query_row("SELECT title FROM games", [], |row| row.get::<_, String>(0)).unwrap(), "Current");
    }
}
