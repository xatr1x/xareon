import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types/settings";

/** Thin typed wrapper around the Tauri `get_settings` / `update_settings` commands. */
export const settingsApi = {
  get(): Promise<Settings> {
    return invoke<Settings>("get_settings");
  },
  update(settings: Settings): Promise<Settings> {
    return invoke<Settings>("update_settings", { settings });
  },
  suspendPlayTrackingShortcut(): Promise<void> {
    return invoke<void>("suspend_play_tracking_shortcut");
  },
  resumePlayTrackingShortcut(): Promise<void> {
    return invoke<void>("resume_play_tracking_shortcut");
  },
};
