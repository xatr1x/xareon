use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use crate::domain::play_session::{DailyPlayTime, PlaySession, PlayTimeTotals, TrackingSource};
use crate::error::AppResult;
use crate::repositories::play_session_repository::{
    PlaySessionRepository, SqlitePlaySessionRepository,
};
use crate::services::play_session_service::PlaySessionService;
use crate::state::AppState;

fn read<T>(state: &State<'_, AppState>, f: impl FnOnce(&PlaySessionService<'_, SqlitePlaySessionRepository<'_>>) -> AppResult<T>) -> AppResult<T> {
    state.db.with_connection(|conn| { let repo = SqlitePlaySessionRepository::new(conn); f(&PlaySessionService::new(&repo)) })
}
fn write<T>(state: &State<'_, AppState>, f: impl FnOnce(&PlaySessionService<'_, SqlitePlaySessionRepository<'_>>) -> AppResult<T>) -> AppResult<T> {
    state.db.with_connection(|conn| { let tx = conn.unchecked_transaction()?; let result = { let repo = SqlitePlaySessionRepository::new(&tx); f(&PlaySessionService::new(&repo))? }; tx.commit()?; Ok(result) })
}

#[tauri::command] pub fn get_active_play_session(state: State<'_, AppState>) -> AppResult<Option<PlaySession>> { read(&state, |s| s.active()) }
#[tauri::command] pub fn get_play_time_totals(state: State<'_, AppState>) -> AppResult<PlayTimeTotals> { read(&state, |s| s.totals()) }
#[tauri::command] pub fn get_game_play_time_today(state: State<'_, AppState>, game_id: i64) -> AppResult<i64> { read(&state, |s| s.game_today(game_id)) }
#[tauri::command] pub fn list_game_daily_play_time(state: State<'_, AppState>, game_id: i64) -> AppResult<Vec<DailyPlayTime>> { read(&state, |s| s.history(game_id)) }
pub(crate) fn set_playing_icon(app: &AppHandle, playing: bool) {
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TrackingChanged {
    pub game_id: Option<i64>,
    pub is_playing: bool,
    pub error: Option<String>,
}

pub(crate) fn toggle_from_global_shortcut(app: &AppHandle) -> AppResult<TrackingChanged> {
    let state = app.state::<AppState>();
    let changed = state.db.with_connection(|conn| {
        let tx = conn.unchecked_transaction()?;
        let changed = {
            let sessions = SqlitePlaySessionRepository::new(&tx);
            if let Some(active) = sessions.active()? {
                sessions.stop(active.game_id)?;
                TrackingChanged { game_id: Some(active.game_id), is_playing: false, error: None }
            } else if let Some(game_id) = sessions.most_recent_game_id()? {
                sessions.start(game_id)?;
                TrackingChanged { game_id: Some(game_id), is_playing: true, error: None }
            } else {
                TrackingChanged {
                    game_id: None,
                    is_playing: false,
                    error: Some("Open Xareon and start a game once before using the shortcut.".into()),
                }
            }
        };
        tx.commit()?;
        Ok(changed)
    })?;
    set_playing_icon(app, changed.is_playing);
    crate::config::session_indicator::refresh(app);
    let _ = app.emit("play-tracking-changed", &changed);
    Ok(changed)
}

#[tauri::command] pub fn start_play_session(app: AppHandle, state: State<'_, AppState>, game_id: i64) -> AppResult<PlaySession> {
    let session = write(&state, |s| s.start(game_id))?;
    set_playing_icon(&app, true);
    crate::config::session_indicator::refresh(&app);
    Ok(session)
}
#[tauri::command] pub fn heartbeat_play_session(state: State<'_, AppState>, game_id: i64) -> AppResult<PlaySession> { write(&state, |s| s.heartbeat(game_id)) }
#[tauri::command] pub fn stop_play_session(app: AppHandle, state: State<'_, AppState>, game_id: i64) -> AppResult<()> {
    let source = read(&state, |s| s.active())?.filter(|session| session.game_id == game_id).map(|session| session.tracking_source);
    write(&state, |s| s.stop(game_id))?;
    if source == Some(TrackingSource::Automatic) {
        app.state::<crate::config::automatic_tracking::AutomaticTrackingRuntime>().suppress(game_id);
    }
    set_playing_icon(&app, false);
    crate::config::session_indicator::refresh(&app);
    Ok(())
}
