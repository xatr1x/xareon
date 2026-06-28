/** Formatting helpers for values coming from the backend. */

/**
 * Backend timestamps are SQLite `datetime('now')` strings in UTC
 * (`YYYY-MM-DD HH:MM:SS`). Render them in the user's locale, date + time.
 */
export function formatDateTime(value: string): string {
  const date = new Date(`${value.replace(" ", "T")}Z`);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** Format an optional ISO date (`YYYY-MM-DD`) for display, or a dash when empty. */
export function formatDate(value: string | null): string {
  if (!value) return "—";
  const date = new Date(`${value}T00:00:00`);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleDateString(undefined, { year: "numeric", month: "short", day: "numeric" });
}
