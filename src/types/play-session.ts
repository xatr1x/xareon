export interface PlaySession {
  gameId: number;
  startedAt: string;
  lastActivityAt: string;
  trackingSource: "manual" | "automatic";
}

export interface DailyPlayTime {
  gameId: number;
  playDate: string;
  durationSeconds: number;
  manualSeconds: number;
  automaticSeconds: number;
  sessionsCount: number;
  firstStartedAt: string;
  lastEndedAt: string;
}

/** Completed-session play time over recent calendar windows, from the backend. */
export interface PlayTimeTotals {
  todaySeconds: number;
  weekSeconds: number;
}
