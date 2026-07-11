import { invoke } from "@tauri-apps/api/core";
import type { PlaySession } from "../types/play-session";

export const playSessionsApi = {
  active: (): Promise<PlaySession | null> => invoke("get_active_play_session"),
  start: (gameId: number): Promise<PlaySession> => invoke("start_play_session", { gameId }),
  heartbeat: (gameId: number): Promise<PlaySession> => invoke("heartbeat_play_session", { gameId }),
  stop: (gameId: number): Promise<void> => invoke("stop_play_session", { gameId }),
};
