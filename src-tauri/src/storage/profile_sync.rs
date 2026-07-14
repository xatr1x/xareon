use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{backup::Backup, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::db::migrations;
use crate::error::{AppError, AppResult};

const BACKUP_FILE: &str = "xareon-backup.sqlite";
const MANIFEST_FILE: &str = "xareon-backup.json";
const LOCAL_STATE_FILE: &str = "profile-sync.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub backup_id: String,
    pub created_at: i64,
    pub platform: String,
    pub schema_version: i64,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalSyncState {
    folder: Option<PathBuf>,
    last_upload_at: Option<i64>,
    last_restore_at: Option<i64>,
    baseline_sha256: Option<String>,
    baseline_backup_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSyncInfo {
    pub folder: Option<String>,
    pub status: SyncStatus,
    pub status_detail: Option<String>,
    pub last_upload_at: Option<i64>,
    pub last_restore_at: Option<i64>,
    pub backup_created_at: Option<i64>,
    pub backup_platform: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SyncStatus {
    FolderNotSelected,
    BackupUnavailable,
    UpToDate,
    LocalNewer,
    BackupNewer,
    Conflict,
    InvalidBackup,
}

pub struct ProfileSyncStorage {
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl ProfileSyncStorage {
    pub fn new(config_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            config_dir,
            data_dir,
        }
    }

    pub fn set_folder(&self, folder: PathBuf) -> AppResult<()> {
        if !folder.is_dir() {
            return Err(AppError::Validation(
                "selected sync folder does not exist".into(),
            ));
        }
        let mut state = self.read_state()?;
        if state.folder.as_ref() != Some(&folder) {
            state.folder = Some(folder);
            state.baseline_sha256 = None;
            state.baseline_backup_id = None;
        }
        self.write_state(&state)
    }

    pub fn info(&self, connection: &Connection) -> AppResult<ProfileSyncInfo> {
        let state = self.read_state()?;
        let Some(folder) = state.folder.as_ref() else {
            return Ok(info_from_state(
                &state,
                SyncStatus::FolderNotSelected,
                None,
                None,
            ));
        };
        if !folder.is_dir() {
            return Ok(info_from_state(
                &state,
                SyncStatus::BackupUnavailable,
                Some("The selected folder is unavailable.".into()),
                None,
            ));
        }

        let manifest = match self.read_and_validate_backup(folder) {
            Ok(manifest) => manifest,
            Err(AppError::Validation(message)) if message == "backup is not available" => {
                return Ok(info_from_state(
                    &state,
                    SyncStatus::BackupUnavailable,
                    None,
                    None,
                ));
            }
            Err(error) => {
                return Ok(info_from_state(
                    &state,
                    SyncStatus::InvalidBackup,
                    Some(error.to_string()),
                    None,
                ));
            }
        };

        let local_temp = self.temporary_path("local-status", "sqlite")?;
        snapshot(connection, &local_temp)?;
        let local_hash = hash_file(&local_temp);
        let _ = fs::remove_file(&local_temp);
        let local_hash = local_hash?;

        let status = if local_hash == manifest.sha256 {
            SyncStatus::UpToDate
        } else if let Some(baseline) = state.baseline_sha256.as_deref() {
            match (local_hash == baseline, manifest.sha256 == baseline) {
                (true, false) => SyncStatus::BackupNewer,
                (false, true) => SyncStatus::LocalNewer,
                (false, false) => SyncStatus::Conflict,
                (true, true) => SyncStatus::UpToDate,
            }
        } else {
            SyncStatus::Conflict
        };

        Ok(info_from_state(&state, status, None, Some(&manifest)))
    }

    pub fn upload(&self, connection: &Connection) -> AppResult<BackupManifest> {
        let mut state = self.read_state()?;
        let folder = selected_folder(&state)?;
        fs::create_dir_all(&folder)?;

        let snapshot_temp = folder.join(".xareon-backup.sqlite.tmp");
        let manifest_temp = folder.join(".xareon-backup.json.tmp");
        remove_if_exists(&snapshot_temp)?;
        remove_if_exists(&manifest_temp)?;
        snapshot(connection, &snapshot_temp)?;

        let sha256 = hash_file(&snapshot_temp)?;
        let created_at = now()?;
        let schema_version = database_version(&snapshot_temp)?;
        let manifest = BackupManifest {
            backup_id: format!("{created_at}-{}", &sha256[..12]),
            created_at,
            platform: std::env::consts::OS.to_string(),
            schema_version,
            size_bytes: fs::metadata(&snapshot_temp)?.len(),
            sha256: sha256.clone(),
        };
        write_json(&manifest_temp, &manifest)?;

        replace_file(&snapshot_temp, &folder.join(BACKUP_FILE))?;
        replace_file(&manifest_temp, &folder.join(MANIFEST_FILE))?;

        state.last_upload_at = Some(created_at);
        state.baseline_sha256 = Some(sha256);
        state.baseline_backup_id = Some(manifest.backup_id.clone());
        self.write_state(&state)?;
        Ok(manifest)
    }

    pub fn restore_into(&self, destination: &mut Connection) -> AppResult<BackupManifest> {
        let mut state = self.read_state()?;
        let folder = selected_folder(&state)?;
        let manifest = self.read_and_validate_backup(&folder)?;

        let safety_dir = self.data_dir.join("backups");
        fs::create_dir_all(&safety_dir)?;
        let safety_path = safety_dir.join(format!("pre-restore-{}.sqlite", now()?));
        snapshot(destination, &safety_path)?;

        let source = Connection::open(folder.join(BACKUP_FILE))?;
        let backup = Backup::new(&source, destination)?;
        backup.run_to_completion(64, Duration::from_millis(10), None)?;
        drop(backup);
        destination.execute_batch("PRAGMA foreign_keys = ON;")?;

        state.last_restore_at = Some(now()?);
        state.baseline_sha256 = Some(manifest.sha256.clone());
        state.baseline_backup_id = Some(manifest.backup_id.clone());
        self.write_state(&state)?;
        Ok(manifest)
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("xareon.db")
    }

    fn read_and_validate_backup(&self, folder: &Path) -> AppResult<BackupManifest> {
        let backup_path = folder.join(BACKUP_FILE);
        let manifest_path = folder.join(MANIFEST_FILE);
        if !backup_path.is_file() || !manifest_path.is_file() {
            return Err(AppError::Validation("backup is not available".into()));
        }
        let manifest: BackupManifest = serde_json::from_slice(&fs::read(manifest_path)?)?;
        let size = fs::metadata(&backup_path)?.len();
        if size != manifest.size_bytes || hash_file(&backup_path)? != manifest.sha256 {
            return Err(AppError::Validation(
                "backup checksum does not match its manifest".into(),
            ));
        }
        let connection = Connection::open(&backup_path)?;
        let integrity: String =
            connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity != "ok" {
            return Err(AppError::Validation(format!(
                "backup integrity check failed: {integrity}"
            )));
        }
        let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version != manifest.schema_version || version > migrations::current_version() {
            return Err(AppError::Validation(
                "backup schema version is not supported".into(),
            ));
        }
        Ok(manifest)
    }

    fn read_state(&self) -> AppResult<LocalSyncState> {
        let path = self.config_dir.join(LOCAL_STATE_FILE);
        if !path.exists() {
            return Ok(LocalSyncState::default());
        }
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }

    fn write_state(&self, state: &LocalSyncState) -> AppResult<()> {
        fs::create_dir_all(&self.config_dir)?;
        let temp = self.config_dir.join(format!("{LOCAL_STATE_FILE}.tmp"));
        write_json(&temp, state)?;
        replace_file(&temp, &self.config_dir.join(LOCAL_STATE_FILE))
    }

    fn temporary_path(&self, stem: &str, extension: &str) -> AppResult<PathBuf> {
        fs::create_dir_all(&self.config_dir)?;
        Ok(self
            .config_dir
            .join(format!(".{stem}-{}.{}", now()?, extension)))
    }
}

fn info_from_state(
    state: &LocalSyncState,
    status: SyncStatus,
    status_detail: Option<String>,
    manifest: Option<&BackupManifest>,
) -> ProfileSyncInfo {
    ProfileSyncInfo {
        folder: state
            .folder
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        status,
        status_detail,
        last_upload_at: state.last_upload_at,
        last_restore_at: state.last_restore_at,
        backup_created_at: manifest.map(|value| value.created_at),
        backup_platform: manifest.map(|value| value.platform.clone()),
    }
}

fn selected_folder(state: &LocalSyncState) -> AppResult<PathBuf> {
    state
        .folder
        .clone()
        .ok_or_else(|| AppError::Validation("select a Google Drive folder first".into()))
}

fn snapshot(source: &Connection, destination_path: &Path) -> AppResult<()> {
    remove_if_exists(destination_path)?;
    let mut destination = Connection::open(destination_path)?;
    let backup = Backup::new(source, &mut destination)?;
    backup.run_to_completion(64, Duration::from_millis(10), None)?;
    Ok(())
}

fn database_version(path: &Path) -> AppResult<i64> {
    let connection = Connection::open(path)?;
    Ok(connection.query_row("PRAGMA user_version", [], |row| row.get(0))?)
}

fn hash_file(path: &Path) -> AppResult<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn write_json(path: &Path, value: &impl Serialize) -> AppResult<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.sync_all()?;
    Ok(())
}

fn replace_file(source: &Path, destination: &Path) -> AppResult<()> {
    let previous = destination.with_extension("previous");
    remove_if_exists(&previous)?;
    if destination.exists() {
        fs::rename(destination, &previous)?;
    }
    if let Err(error) = fs::rename(source, destination) {
        if previous.exists() {
            let _ = fs::rename(&previous, destination);
        }
        return Err(error.into());
    }
    remove_if_exists(&previous)?;
    Ok(())
}

fn remove_if_exists(path: &Path) -> AppResult<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn now() -> AppResult<i64> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::Validation("system clock is before Unix epoch".into()))?
        .as_secs();
    i64::try_from(seconds).map_err(|_| AppError::Validation("system time is out of range".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database(value: &str) -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(&format!(
                "CREATE TABLE notes (value TEXT NOT NULL); \
                 INSERT INTO notes VALUES ('{value}'); \
                 PRAGMA user_version = {};",
                migrations::current_version()
            ))
            .unwrap();
        connection
    }

    #[test]
    fn upload_detects_local_changes_and_restore_recovers_backup() {
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("drive");
        fs::create_dir_all(&folder).unwrap();
        let storage = ProfileSyncStorage::new(root.path().join("config"), root.path().join("data"));
        storage.set_folder(folder).unwrap();
        let mut connection = database("original");

        storage.upload(&connection).unwrap();
        assert_eq!(
            storage.info(&connection).unwrap().status,
            SyncStatus::UpToDate
        );

        connection
            .execute("UPDATE notes SET value = 'changed'", [])
            .unwrap();
        assert_eq!(
            storage.info(&connection).unwrap().status,
            SyncStatus::LocalNewer
        );

        storage.restore_into(&mut connection).unwrap();
        let value: String = connection
            .query_row("SELECT value FROM notes", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, "original");
        assert_eq!(
            storage.info(&connection).unwrap().status,
            SyncStatus::UpToDate
        );
        assert!(storage
            .data_dir
            .join("backups")
            .read_dir()
            .unwrap()
            .next()
            .is_some());
    }

    #[test]
    fn another_device_upload_is_reported_as_newer_backup() {
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("drive");
        fs::create_dir_all(&folder).unwrap();
        let first = ProfileSyncStorage::new(
            root.path().join("first-config"),
            root.path().join("first-data"),
        );
        let second = ProfileSyncStorage::new(
            root.path().join("second-config"),
            root.path().join("second-data"),
        );
        first.set_folder(folder.clone()).unwrap();
        second.set_folder(folder).unwrap();
        let first_db = database("first");
        first.upload(&first_db).unwrap();

        let mut second_db = database("first");
        second.restore_into(&mut second_db).unwrap();
        second_db
            .execute("UPDATE notes SET value = 'second'", [])
            .unwrap();
        second.upload(&second_db).unwrap();

        assert_eq!(
            first.info(&first_db).unwrap().status,
            SyncStatus::BackupNewer
        );
    }
}
