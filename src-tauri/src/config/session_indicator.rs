use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::repositories::play_session_repository::{
    PlaySessionRepository, SqlitePlaySessionRepository,
};
use crate::state::AppState;

const TRAY_ID: &str = "active-play-session";

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let icon = tauri::image::Image::from_bytes(include_bytes!("../../icons/tray-playing.png"))?;
    let tray = tauri::tray::TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(false)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if matches!(event, tauri::tray::TrayIconEvent::Click { .. }) {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    tray.set_visible(false)?;

    let handle = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(60));
        heartbeat_and_refresh(&handle);
    });
    Ok(())
}

pub fn refresh(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let state = app.state::<AppState>();
    let summary = state.db.with_connection(|conn| {
        SqlitePlaySessionRepository::new(conn).active_summary()
    });

    match summary {
        Ok(Some(summary)) => {
            let duration = format_duration(summary.elapsed_seconds);
            let _ = tray.set_title(Some(&duration));
            let _ = tray.set_tooltip(Some(format!("{} — {}", summary.game_title, duration)));
            let _ = tray.set_visible(true);
        }
        _ => {
            let _ = tray.set_visible(false);
            let _ = tray.set_title(None::<&str>);
        }
    }
}

fn heartbeat_and_refresh(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _ = state.db.with_connection(|conn| {
        let tx = conn.unchecked_transaction()?;
        {
            let sessions = SqlitePlaySessionRepository::new(&tx);
            if let Some(active) = sessions.active()? {
                sessions.heartbeat(active.game_id)?;
            }
        }
        tx.commit()?;
        Ok(())
    });
    refresh(app);
}

fn format_duration(seconds: i64) -> String {
    let minutes = seconds.max(0) / 60;
    let hours = minutes / 60;
    let remaining = minutes % 60;
    if hours == 0 {
        format!("{minutes}m")
    } else {
        format!("{hours}h {remaining}m")
    }
}
