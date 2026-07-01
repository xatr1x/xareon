import { invoke } from "@tauri-apps/api/core";
import type { Achievement, AchievementUpdate, NewAchievement } from "../types/achievement";

/** Thin typed wrapper around the Tauri achievement commands. */
export const achievementsApi = {
  listForGame(gameId: number): Promise<Achievement[]> {
    return invoke<Achievement[]>("list_achievements", { gameId });
  },
  create(input: NewAchievement): Promise<Achievement> {
    return invoke<Achievement>("create_achievement", { input });
  },
  update(id: number, update: AchievementUpdate): Promise<Achievement> {
    return invoke<Achievement>("update_achievement", { id, update });
  },
  setProgress(id: number, progressCurrent: number): Promise<Achievement> {
    return invoke<Achievement>("set_achievement_progress", { id, progressCurrent });
  },
  complete(id: number): Promise<Achievement> {
    return invoke<Achievement>("complete_achievement", { id });
  },
  reopen(id: number): Promise<Achievement> {
    return invoke<Achievement>("reopen_achievement", { id });
  },
  delete(id: number): Promise<void> {
    return invoke<void>("delete_achievement", { id });
  },
};
