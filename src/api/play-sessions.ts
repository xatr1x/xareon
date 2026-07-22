import { invoke } from "@tauri-apps/api/core";
import type { DailyPlayTime, PlaySession, PlayTimeTotals } from "../types/play-session";

export const playSessionsApi = {
  active: (): Promise<PlaySession | null> => invoke("get_active_play_session"),
  totals: (): Promise<PlayTimeTotals> => invoke("get_play_time_totals"),
  gameTodaySeconds: (gameId: number): Promise<number> =>
    invoke("get_game_play_time_today", { gameId }),
  history: (gameId: number): Promise<DailyPlayTime[]> => invoke("list_game_daily_play_time", { gameId }),
  start: (gameId: number): Promise<PlaySession> => invoke("start_play_session", { gameId }),
  heartbeat: (gameId: number): Promise<PlaySession> => invoke("heartbeat_play_session", { gameId }),
  stop: (gameId: number): Promise<void> => invoke("stop_play_session", { gameId }),
};
