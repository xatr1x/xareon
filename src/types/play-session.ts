export interface PlaySession {
  id: number;
  gameId: number;
  startedAt: string;
  endedAt: string | null;
  lastActivityAt: string;
  durationSeconds: number | null;
}
