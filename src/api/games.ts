import { invoke } from "@tauri-apps/api/core";
import type { Game, GameInput } from "../types/game";

/**
 * Thin typed wrapper around the Tauri `game_*` commands. The UI layer talks to
 * this module only — it never calls `invoke` directly and holds no SQL.
 */
export const gamesApi = {
  list(): Promise<Game[]> {
    return invoke<Game[]>("list_games");
  },
  get(id: number): Promise<Game> {
    return invoke<Game>("get_game", { id });
  },
  create(input: GameInput): Promise<Game> {
    return invoke<Game>("create_game", { input });
  },
  update(id: number, input: GameInput): Promise<Game> {
    return invoke<Game>("update_game", { id, input });
  },
  delete(id: number): Promise<void> {
    return invoke<void>("delete_game", { id });
  },
};
