/** Bucketing for the "play time over time" chart. Mirrors the Rust enum. */
export type StatsGranularity = "week" | "month" | "year";

/** One labelled data point; `value` is seconds (time series) or a count. */
export interface StatBar {
  key: string;
  value: number;
}

export interface StatsSummary {
  totalPlaySeconds: number;
  yearPlaySeconds: number;
  completedCount: number;
  playingCount: number;
  backlogCount: number;
  averageRating: number | null;
}

/** Full Statistics payload, mirroring the Rust `Statistics` struct. */
export interface Statistics {
  summary: StatsSummary;
  daily: StatBar[];
  overTime: StatBar[];
  weekday: StatBar[];
  topGames: StatBar[];
  genres: StatBar[];
  statuses: StatBar[];
  ratings: StatBar[];
  granularity: StatsGranularity;
}
