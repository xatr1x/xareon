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
mod platform;

use tauri::Manager;

use crate::config::device_settings::DeviceSettings;
use crate::db::manager::DatabaseManager;
use crate::repositories::play_session_repository::{
    PlaySessionRepository, SqlitePlaySessionRepository,
};
use crate::state::AppState;

/// Build and run the Tauri application: resolve the data directory, open the
/// database (running migrations), register state and commands, then start.
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let builder = builder.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, _shortcut, event| {
                use tauri::Emitter;
                use tauri_plugin_global_shortcut::ShortcutState;

                if event.state() == ShortcutState::Pressed {
                    if let Err(error) =
                        commands::play_session_commands::toggle_from_global_shortcut(app)
                    {
                        let changed = commands::play_session_commands::TrackingChanged {
                            game_id: None,
                            is_playing: false,
                            error: Some(error.to_string()),
                        };
                        let _ = app.emit("play-tracking-changed", changed);
                    }
                }
            })
            .build(),
    );

    builder
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
            app.manage(crate::config::automatic_tracking::AutomaticTrackingRuntime::default());
            crate::config::session_indicator::setup(app.handle())?;
            crate::config::automatic_tracking::setup(app.handle());
            let config_dir = app.path().app_config_dir()?;
            let mut device_settings = DeviceSettings::load(&config_dir)?;
            match crate::config::global_shortcut::replace(
                app.handle(),
                None,
                device_settings.play_tracking_shortcut.as_deref(),
            ) {
                Ok(()) => device_settings.shortcut_registration_error = None,
                Err(error) => {
                    // A platform-reserved or occupied shortcut must never prevent
                    // the application from starting. Settings shows the error.
                    device_settings.shortcut_registration_error = Some(error.to_string());
                }
            }
            device_settings.save(&config_dir)?;
            // Set the base (non-playing) Dock icon at launch so it always shows,
            // not only after a session starts.
            commands::play_session_commands::set_playing_icon(app.handle(), false);
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
                            sessions.stop_with_reason(active.game_id, crate::domain::play_session::SessionEndReason::Shutdown)?;
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
            commands::settings_commands::suspend_play_tracking_shortcut,
            commands::settings_commands::resume_play_tracking_shortcut,
            commands::profile_sync_commands::choose_profile_sync_folder,
            commands::profile_sync_commands::get_profile_sync_info,
            commands::profile_sync_commands::upload_profile_backup,
            commands::profile_sync_commands::restore_profile_backup,
            commands::profile_sync_commands::open_database_folder,
            commands::play_session_commands::get_active_play_session,
            commands::play_session_commands::get_play_time_totals,
            commands::play_session_commands::get_game_play_time_today,
            commands::play_session_commands::list_game_play_sessions,
            commands::play_session_commands::start_play_session,
            commands::play_session_commands::heartbeat_play_session,
            commands::play_session_commands::stop_play_session,
            commands::statistics_commands::get_statistics,
            commands::automatic_tracking_commands::get_platform_capabilities,
            commands::automatic_tracking_commands::list_running_game_processes,
            commands::automatic_tracking_commands::list_game_executable_bindings,
            commands::automatic_tracking_commands::add_game_executable_binding,
            commands::automatic_tracking_commands::delete_game_executable_binding,
            commands::automatic_tracking_commands::set_automatic_tracking_enabled,
            commands::automatic_tracking_commands::get_automatic_tracking_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Xareon");
}
