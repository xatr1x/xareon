/**
 * Frontend mirror of the Rust domain types. Keep these field names in sync with
 * `src-tauri/src/domain/game.rs` (serde uses snake_case on the wire).
 */

export const GAME_STATUSES = [
  "planned",
  "playing",
  "paused",
  "completed",
  "completed_100",
  "dropped",
] as const;

export type GameStatus = (typeof GAME_STATUSES)[number];

export const STATUS_LABELS: Record<GameStatus, string> = {
  planned: "Planned",
  playing: "Playing",
  paused: "Paused",
  completed: "Completed",
  completed_100: "Completed 100%",
  dropped: "Dropped",
};

export interface Game {
  id: number;
  title: string;
  genre: string | null;
  platform: string | null;
  developer: string | null;
  publisher: string | null;
  releaseYear: number | null;
  startedAt: string | null;
  finishedAt: string | null;
  status: GameStatus;
  rating: number | null;
  coverPath: string | null;
  createdAt: string;
  updatedAt: string;
}

/** Payload for creating or updating a game (everything except server-managed fields). */
export interface GameInput {
  title: string;
  genre: string | null;
  platform: string | null;
  developer: string | null;
  publisher: string | null;
  releaseYear: number | null;
  startedAt: string | null;
  finishedAt: string | null;
  status: GameStatus;
  rating: number | null;
  coverPath: string | null;
}
