use crate::db::manager::DatabaseManager;

/// Shared application state managed by Tauri. The database is exposed only
/// through [`DatabaseManager`]; nothing depends on how the connection is held.
pub struct AppState {
    pub db: DatabaseManager,
}
