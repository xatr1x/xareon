import { invoke } from "@tauri-apps/api/core";
import type { Game, GameInput, GameQuery } from "../types/game";
import type { Genre } from "../types/genre";

/**
 * Thin typed wrapper around the Tauri `game_*` / `list_genres` commands. The UI
 * layer talks to this module only — it never calls `invoke` directly and holds
 * no SQL.
 */
export const gamesApi = {
  list(query: GameQuery = {}): Promise<Game[]> {
    return invoke<Game[]>("list_games", { query });
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

export const genresApi = {
  list(): Promise<Genre[]> {
    return invoke<Genre[]>("list_genres");
  },
};
