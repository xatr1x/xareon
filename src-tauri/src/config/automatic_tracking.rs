use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::domain::automatic_tracking::AutomaticTrackingState;
use crate::domain::play_session::{SessionEndReason, TrackingSource};
use crate::repositories::game_process_repository::{GameProcessRepository, SqliteGameProcessRepository};
use crate::repositories::play_session_repository::{PlaySessionRepository, SqlitePlaySessionRepository};
use crate::state::AppState;

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const AFK_GRACE: Duration = Duration::from_secs(10 * 60);

#[derive(Default)]
struct GameRuntime { state: Option<AutomaticTrackingState>, last_input_tick: u32, inactive_since: Option<Instant>, was_foreground: bool }

#[derive(Default)]
pub struct AutomaticTrackingRuntime {
    games: Mutex<HashMap<i64, GameRuntime>>,
    suppressed: Mutex<HashSet<i64>>,
}

impl AutomaticTrackingRuntime {
    pub fn suppress(&self, game_id: i64) { self.suppressed.lock().expect("automatic tracking suppression poisoned").insert(game_id); }
    pub fn state(&self, game_id: i64) -> Option<AutomaticTrackingState> { self.games.lock().ok()?.get(&game_id).and_then(|g| g.state) }
}

#[cfg(target_os = "windows")]
pub fn setup(app: &AppHandle) {
    let handle = app.clone();
    std::thread::spawn(move || {
        let mut last_heartbeat = Instant::now();
        loop {
            tick(&handle);
            if last_heartbeat.elapsed() >= Duration::from_secs(60) {
                crate::config::session_indicator::heartbeat_and_refresh(&handle);
                last_heartbeat = Instant::now();
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    });
}

#[cfg(not(target_os = "windows"))]
pub fn setup(_app: &AppHandle) {}

#[cfg(target_os = "windows")]
fn tick(app: &AppHandle) {
    use crate::platform::windows::{last_input_tick, observe_processes};

    let observations = observe_processes();
    let input_tick = last_input_tick();
    let state = app.state::<AppState>();
    let bindings = match state.db.with_connection(|conn| SqliteGameProcessRepository::new(conn).enabled_bindings()) { Ok(v) => v, Err(_) => return };
    let paths: HashMap<String, Vec<_>> = observations.into_iter().fold(HashMap::new(), |mut map, observation| {
        map.entry(normalize(&observation.process.executable_path)).or_default().push(observation); map
    });
    let mut by_game: HashMap<i64, Vec<_>> = HashMap::new();
    for binding in bindings { if let Some(items) = paths.get(&binding.executable_normalized) { by_game.entry(binding.game_id).or_default().extend(items.iter()); } }

    let runtime = app.state::<AutomaticTrackingRuntime>();
    let mut games = match runtime.games.lock() { Ok(v) => v, Err(_) => return };
    let mut suppressed = match runtime.suppressed.lock() { Ok(v) => v, Err(_) => return };
    let configured: HashSet<i64> = by_game.keys().copied().collect();
    suppressed.retain(|game_id| configured.contains(game_id));

    let active = state.db.with_connection(|conn| SqlitePlaySessionRepository::new(conn).active()).ok().flatten();
    let mut changed = false;
    let mut changed_game_id = None;
    for (game_id, candidates) in &by_game {
        let game = games.entry(*game_id).or_default();
        if suppressed.contains(game_id) { game.state = Some(AutomaticTrackingState::Suppressed); continue; }
        let foreground = candidates.iter().any(|o| o.is_foreground && !o.is_minimized);
        let new_input = foreground && game.was_foreground && game.last_input_tick != 0 && input_tick != 0 && input_tick != game.last_input_tick;
        if foreground { game.last_input_tick = input_tick; }
        game.was_foreground = foreground;
        let owns_automatic = active.as_ref().is_some_and(|s| s.game_id == *game_id && s.tracking_source == TrackingSource::Automatic);
        if owns_automatic {
            if new_input {
                game.inactive_since = None;
                game.state = Some(AutomaticTrackingState::Tracking);
            } else {
                let since = game.inactive_since.get_or_insert_with(Instant::now);
                if since.elapsed() >= AFK_GRACE {
                    if stop_automatic(app, *game_id, SessionEndReason::Afk) { changed = true; changed_game_id = Some(*game_id); }
                    game.state = Some(AutomaticTrackingState::Afk);
                    game.inactive_since = None;
                }
            }
        } else if active.is_none() && new_input {
            if start_automatic(app, *game_id) { changed = true; changed_game_id = Some(*game_id); game.state = Some(AutomaticTrackingState::Tracking); }
        } else {
            game.state = Some(if game.state.is_some_and(|s| matches!(s, AutomaticTrackingState::Afk)) { AutomaticTrackingState::Afk } else { AutomaticTrackingState::WaitingForActivity });
        }
    }

    if let Some(session) = active.filter(|s| s.tracking_source == TrackingSource::Automatic) {
        if !by_game.contains_key(&session.game_id) {
            if stop_automatic(app, session.game_id, SessionEndReason::ProcessExit) { changed = true; changed_game_id = Some(session.game_id); }
            games.entry(session.game_id).or_default().state = Some(AutomaticTrackingState::ProcessNotRunning);
        }
    }
    for (game_id, game) in games.iter_mut() { if !by_game.contains_key(game_id) { game.state = Some(AutomaticTrackingState::ProcessNotRunning); game.inactive_since = None; game.was_foreground = false; } }
    drop(suppressed); drop(games);
    if changed { refresh_tracking_ui(app, changed_game_id); }
}

#[cfg(target_os = "windows")]
fn start_automatic(app: &AppHandle, game_id: i64) -> bool {
    let state = app.state::<AppState>();
    state.db.with_connection(|conn| { let tx = conn.unchecked_transaction()?; SqlitePlaySessionRepository::new(&tx).start_automatic(game_id)?; tx.commit()?; Ok(()) }).is_ok()
}

#[cfg(target_os = "windows")]
fn stop_automatic(app: &AppHandle, game_id: i64, reason: SessionEndReason) -> bool {
    let state = app.state::<AppState>();
    state.db.with_connection(|conn| { let tx = conn.unchecked_transaction()?; SqlitePlaySessionRepository::new(&tx).stop_with_reason(game_id, reason)?; tx.commit()?; Ok(()) }).is_ok()
}

#[cfg(target_os = "windows")]
fn refresh_tracking_ui(app: &AppHandle, game_id: Option<i64>) {
    let playing = app.state::<AppState>().db.with_connection(|conn| SqlitePlaySessionRepository::new(conn).active()).ok().flatten().is_some();
    crate::commands::play_session_commands::set_playing_icon(app, playing);
    crate::config::session_indicator::refresh(app);
    let _ = app.emit("play-tracking-changed", serde_json::json!({ "gameId": game_id, "isPlaying": playing, "error": null }));
}

fn normalize(path: &str) -> String { path.trim().replace('/', "\\").to_lowercase() }
