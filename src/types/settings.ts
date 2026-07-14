/** Mirror of the Rust settings domain type (camelCase on the wire). */

export interface Settings {
  userIdentifier: string | null;
  playTrackingShortcut: string | null;
  playTrackingShortcutError?: string | null;
}

export type ProfileSyncStatus =
  | "folderNotSelected"
  | "backupUnavailable"
  | "upToDate"
  | "localNewer"
  | "backupNewer"
  | "conflict"
  | "invalidBackup";

export interface ProfileSyncInfo {
  folder: string | null;
  status: ProfileSyncStatus;
  statusDetail: string | null;
  lastUploadAt: number | null;
  lastRestoreAt: number | null;
  backupCreatedAt: number | null;
  backupPlatform: string | null;
}
