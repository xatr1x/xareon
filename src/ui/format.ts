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

function parseCalendarDate(value: string): Date | null {
  const date = new Date(`${value}T00:00:00`);
  return Number.isNaN(date.getTime()) ? null : date;
}

function calendarDaysBetween(from: Date, to: Date): number {
  const fromDay = Date.UTC(from.getFullYear(), from.getMonth(), from.getDate());
  const toDay = Date.UTC(to.getFullYear(), to.getMonth(), to.getDate());
  return Math.round((toDay - fromDay) / 86_400_000);
}

/** Formats a calendar duration compactly, for example `1 mo, 3 d`. */
export function formatPlayDuration(startedAt: string, finishedAt: string): string {
  const start = parseCalendarDate(startedAt);
  const finish = parseCalendarDate(finishedAt);
  if (!start || !finish) return "—";
  if (calendarDaysBetween(start, finish) <= 0) return "Same day";

  let months = (finish.getFullYear() - start.getFullYear()) * 12 + finish.getMonth() - start.getMonth();
  let monthMark = new Date(start.getFullYear(), start.getMonth() + months, start.getDate());
  if (monthMark > finish) {
    months -= 1;
    monthMark = new Date(start.getFullYear(), start.getMonth() + months, start.getDate());
  }

  const days = calendarDaysBetween(monthMark, finish);
  const parts: string[] = [];
  if (months > 0) parts.push(`${months} mo`);
  if (days > 0) parts.push(`${days} d`);
  return parts.join(", ") || "Same day";
}

/** Formats a past start date relative to today, for example `Yesterday` or `2 wks ago`. */
export function formatTimeSince(startedAt: string): string {
  const start = parseCalendarDate(startedAt);
  if (!start) return "—";

  const days = calendarDaysBetween(start, new Date());
  if (days <= 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 7) return `${days} days ago`;

  const weeks = Math.floor(days / 7);
  if (days < 30) return `${weeks} wk${weeks === 1 ? "" : "s"} ago`;

  const months = Math.floor(days / 30);
  return `${months} mo${months === 1 ? "" : "s"} ago`;
}

export function parseBackendDateTime(value: string): Date {
  return new Date(`${value.replace(" ", "T")}Z`);
}

/** Compact accumulated play time, rounded down to completed minutes. */
export function formatTrackedDuration(seconds: number): string {
  const minutes = Math.floor(Math.max(0, seconds) / 60);
  const hours = Math.floor(minutes / 60);
  const remaining = minutes % 60;
  if (hours === 0) return `${minutes}m`;
  return remaining === 0 ? `${hours}h` : `${hours}h ${remaining}m`;
}

export function formatSessionTimer(startedAt: string): string {
  const elapsed = Math.max(0, Math.floor((Date.now() - parseBackendDateTime(startedAt).getTime()) / 1000));
  const hours = Math.floor(elapsed / 3600);
  const minutes = Math.floor((elapsed % 3600) / 60);
  const seconds = elapsed % 60;
  return [hours, minutes, seconds].map((part) => String(part).padStart(2, "0")).join(":");
}

/** Steam-like relative time for a past backend timestamp. */
export function formatRelativeTime(value: string, now = Date.now()): string {
  const timestamp = parseBackendDateTime(value).getTime();
  if (Number.isNaN(timestamp)) return value;

  const elapsedSeconds = Math.max(0, Math.floor((now - timestamp) / 1000));
  if (elapsedSeconds < 60) return "just now";

  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 365 * 24 * 60 * 60],
    ["month", 30 * 24 * 60 * 60],
    ["week", 7 * 24 * 60 * 60],
    ["day", 24 * 60 * 60],
    ["hour", 60 * 60],
    ["minute", 60],
  ];
  const [unit, seconds] = units.find(([, size]) => elapsedSeconds >= size) ?? ["minute", 60];
  const amount = Math.floor(elapsedSeconds / seconds);
  return new Intl.RelativeTimeFormat("en", { numeric: "always" }).format(-amount, unit);
}
