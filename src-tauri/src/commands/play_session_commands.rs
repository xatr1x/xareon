use tauri::{AppHandle, State};
#[cfg(not(target_os = "macos"))]
use tauri::Manager;
use crate::domain::play_session::PlaySession;
use crate::error::AppResult;
use crate::repositories::play_session_repository::SqlitePlaySessionRepository;
use crate::services::play_session_service::PlaySessionService;
use crate::state::AppState;

fn read<T>(state: &State<'_, AppState>, f: impl FnOnce(&PlaySessionService<'_, SqlitePlaySessionRepository<'_>>) -> AppResult<T>) -> AppResult<T> {
    state.db.with_connection(|conn| { let repo = SqlitePlaySessionRepository::new(conn); f(&PlaySessionService::new(&repo)) })
}
fn write<T>(state: &State<'_, AppState>, f: impl FnOnce(&PlaySessionService<'_, SqlitePlaySessionRepository<'_>>) -> AppResult<T>) -> AppResult<T> {
    state.db.with_connection(|conn| { let tx = conn.unchecked_transaction()?; let result = { let repo = SqlitePlaySessionRepository::new(&tx); f(&PlaySessionService::new(&repo))? }; tx.commit()?; Ok(result) })
}

#[tauri::command] pub fn get_active_play_session(state: State<'_, AppState>) -> AppResult<Option<PlaySession>> { read(&state, |s| s.active()) }
fn set_playing_icon(app: &AppHandle, playing: bool) {
    let bytes = if playing {
        include_bytes!("../../icons/icon-playing.png").as_slice()
    } else {
        include_bytes!("../../icons/icon-source.png").as_slice()
    };

    #[cfg(target_os = "macos")]
    {
        use objc2::{AnyThread, MainThreadMarker};
        use objc2_app_kit::{NSApplication, NSImage};
        use objc2_foundation::NSData;

        let _ = app.run_on_main_thread(move || {
            let Some(mtm) = MainThreadMarker::new() else { return };
            let data = unsafe { NSData::dataWithBytes_length(bytes.as_ptr().cast(), bytes.len()) };
            let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) else { return };
            let application = NSApplication::sharedApplication(mtm);
            unsafe { application.setApplicationIconImage(Some(&image)) };
        });
    }

    #[cfg(not(target_os = "macos"))]
    if let (Ok(icon), Some(window)) = (
        tauri::image::Image::from_bytes(bytes),
        app.get_webview_window("main"),
    ) {
        let _ = window.set_icon(icon);
    }
}

#[tauri::command] pub fn start_play_session(app: AppHandle, state: State<'_, AppState>, game_id: i64) -> AppResult<PlaySession> {
    let session = write(&state, |s| s.start(game_id))?;
    set_playing_icon(&app, true);
    Ok(session)
}
#[tauri::command] pub fn heartbeat_play_session(state: State<'_, AppState>, game_id: i64) -> AppResult<PlaySession> { write(&state, |s| s.heartbeat(game_id)) }
#[tauri::command] pub fn stop_play_session(app: AppHandle, state: State<'_, AppState>, game_id: i64) -> AppResult<()> {
    write(&state, |s| s.stop(game_id))?;
    set_playing_icon(&app, false);
    Ok(())
}
