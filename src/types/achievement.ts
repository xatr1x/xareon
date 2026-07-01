/** Mirror of the Rust achievement domain types (camelCase on the wire). */

export type AchievementStatus = "planned" | "in_progress" | "completed";

export const ACHIEVEMENT_STATUSES: AchievementStatus[] = ["planned", "in_progress", "completed"];

export const ACHIEVEMENT_STATUS_LABELS: Record<AchievementStatus, string> = {
  planned: "Planned",
  in_progress: "In progress",
  completed: "Completed",
};

export interface Achievement {
  id: number;
  gameId: number;
  title: string;
  description: string | null;
  category: string | null;
  status: AchievementStatus;
  progressCurrent: number | null;
  progressTarget: number | null;
  progressUnit: string | null;
  completedAt: string | null;
  isHidden: boolean;
  displayOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface NewAchievement {
  gameId: number;
  title: string;
  description: string | null;
  category: string | null;
  status: AchievementStatus;
  progressCurrent: number | null;
  progressTarget: number | null;
  progressUnit: string | null;
  completedAt: string | null;
  isHidden: boolean;
  displayOrder: number | null;
}

export type AchievementUpdate = Omit<NewAchievement, "gameId" | "displayOrder"> & {
  displayOrder: number;
};
