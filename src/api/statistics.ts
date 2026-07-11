import { invoke } from "@tauri-apps/api/core";
import type { Statistics, StatsGranularity } from "../types/statistics";

export const statisticsApi = {
  get: (granularity: StatsGranularity): Promise<Statistics> =>
    invoke("get_statistics", { granularity }),
};
