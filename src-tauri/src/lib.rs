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
use crate::repositories::play_session_repository::{PlaySessionRepository, SqlitePlaySessionRepository};

/// Build and run the Tauri application: resolve the data directory, open the
/// database (running migrations), register state and commands, then start.
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = db::connection::open(&data_dir.join("xareon.db"))?;
            {
                let tx = conn.unchecked_transaction()?;
                SqlitePlaySessionRepository::new(&tx).recover_interrupted()?;
                tx.commit()?;
            }
            app.manage(AppState {
                db: DatabaseManager::new(conn),
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                let state = window.state::<AppState>();
                let _ = state.db.with_connection(|conn| {
                    let tx = conn.unchecked_transaction()?;
                    {
                        let sessions = SqlitePlaySessionRepository::new(&tx);
                        if let Some(active) = sessions.active()? {
                            sessions.stop(active.game_id)?;
                        }
                    }
                    tx.commit()?;
                    Ok(())
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::achievement_commands::list_achievements,
            commands::achievement_commands::create_achievement,
            commands::achievement_commands::update_achievement,
            commands::achievement_commands::set_achievement_progress,
            commands::achievement_commands::complete_achievement,
            commands::achievement_commands::reopen_achievement,
            commands::achievement_commands::delete_achievement,
            commands::game_commands::list_games,
            commands::game_commands::get_game,
            commands::game_commands::create_game,
            commands::game_commands::update_game,
            commands::game_commands::delete_game,
            commands::genre_commands::list_genres,
            commands::journal_commands::list_journal_entries,
            commands::journal_commands::create_journal_entry,
            commands::journal_commands::update_journal_entry,
            commands::journal_commands::delete_journal_entry,
            commands::settings_commands::get_settings,
            commands::settings_commands::update_settings,
            commands::play_session_commands::get_active_play_session,
            commands::play_session_commands::start_play_session,
            commands::play_session_commands::heartbeat_play_session,
            commands::play_session_commands::stop_play_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Xareon");
}
