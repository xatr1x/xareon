use tauri::AppHandle;

use crate::error::{AppError, AppResult};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use tauri_plugin_global_shortcut::GlobalShortcutExt;

/// Replace the registered Play/Stop shortcut. If the new accelerator is invalid
/// or occupied, restore the previous registration and keep the stored setting.
pub fn replace(app: &AppHandle, previous: Option<&str>, next: Option<&str>) -> AppResult<()> {
    if previous == next {
        return Ok(());
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        if let Some(shortcut) = previous {
            let _ = app.global_shortcut().unregister(shortcut);
        }

        if let Some(shortcut) = next {
            if let Err(error) = app.global_shortcut().register(shortcut) {
                if let Some(old) = previous {
                    let _ = app.global_shortcut().register(old);
                }
                return Err(AppError::Validation(format!(
                    "shortcut is invalid or already in use: {error}"
                )));
            }
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = (app, previous, next);

    Ok(())
}
