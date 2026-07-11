/** Mirror of the Rust settings domain type (camelCase on the wire). */

export interface Settings {
  userIdentifier: string | null;
  googleDriveFolder: string | null;
  playTrackingShortcut: string | null;
}
