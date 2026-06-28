//! Xareon backend library. `main.rs` simply calls [`run`].
//!
//! Layering (outer depends on inner): commands → services → repositories → db,
//! with `domain` holding shared models and `error` the shared error type.

mod commands;
mod config;
mod db;
mod domain;
mod error;
mod events;
mod repositories;
mod services;
mod state;
mod storage;
mod validation;

use tauri::Manager;

use crate::db::manager::DatabaseManager;
use crate::state::AppState;

/// Build and run the Tauri application: resolve the data directory, open the
/// database (running migrations), register state and commands, then start.
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = db::connection::open(&data_dir.join("xareon.db"))?;
            app.manage(AppState {
                db: DatabaseManager::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::game_commands::list_games,
            commands::game_commands::get_game,
            commands::game_commands::create_game,
            commands::game_commands::update_game,
            commands::game_commands::delete_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Xareon");
}
