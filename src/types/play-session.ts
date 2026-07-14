export interface PlaySession {
  id: number;
  gameId: number;
  startedAt: string;
  endedAt: string | null;
  lastActivityAt: string;
  durationSeconds: number | null;
  trackingSource: "manual" | "automatic";
  endedReason: "manual" | "process_exit" | "afk" | "shutdown" | "recovery" | null;
}

/** Completed-session play time over recent calendar windows, from the backend. */
export interface PlayTimeTotals {
  todaySeconds: number;
  weekSeconds: number;
}
