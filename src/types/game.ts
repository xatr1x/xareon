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
  genres: string[];
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
  genres: string[];
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

export type GenreMatch = "any" | "all";

export const GAME_SORTS = [
  "default",
  "title",
  "started_at",
  "finished_at",
  "release_year",
  "rating",
  "status",
] as const;
export type GameSort = (typeof GAME_SORTS)[number];

export const SORT_LABELS: Record<GameSort, string> = {
  default: "Default (Playing first)",
  title: "Title",
  started_at: "Started",
  finished_at: "Finished",
  release_year: "Release year",
  rating: "Rating",
  status: "Status",
};

export type SortDirection = "asc" | "desc";

/**
 * Flexible game browser query. Mirrors the Rust `GameQuery`. Only set the fields
 * you want to filter on; omit the rest. Combine filters freely.
 */
export interface GameQuery {
  search?: string;
  statuses?: GameStatus[];
  platforms?: string[];
  genres?: string[];
  genreMatch?: GenreMatch;
  releaseYear?: number;
  startedYear?: number;
  finishedYear?: number;
  playedYear?: number;
  minRating?: number;
  maxRating?: number;
  sort?: GameSort;
  direction?: SortDirection;
}
