import { invoke } from "@tauri-apps/api/core";
import type { ProfileSyncInfo, Settings } from "../types/settings";

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
  getProfileSyncInfo(): Promise<ProfileSyncInfo> {
    return invoke<ProfileSyncInfo>("get_profile_sync_info");
  },
  chooseProfileSyncFolder(): Promise<ProfileSyncInfo | null> {
    return invoke<ProfileSyncInfo | null>("choose_profile_sync_folder");
  },
  uploadProfileBackup(): Promise<void> {
    return invoke<void>("upload_profile_backup");
  },
  restoreProfileBackup(): Promise<void> {
    return invoke<void>("restore_profile_backup");
  },
  openDatabaseFolder(): Promise<void> {
    return invoke<void>("open_database_folder");
  },
};
