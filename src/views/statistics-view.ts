import { statisticsApi } from "../api/statistics";
import { clear, el } from "../ui/dom";
import { formatTrackedDuration, startOfLocalWeek } from "../ui/format";
import type { StatBar, Statistics, StatsGranularity } from "../types/statistics";

/** Persisted across navigation so returning to Statistics keeps the granularity. */
let granularity: StatsGranularity = "month";

const HEATMAP_WEEKS = 53;
const DAY_MS = 86_400_000;

const GRANULARITIES: Array<{ id: StatsGranularity; label: string }> = [
  { id: "week", label: "Week" },
  { id: "month", label: "Month" },
  { id: "year", label: "Year" },
];

const WEEKDAYS: Array<{ key: string; label: string }> = [
  { key: "1", label: "Mon" },
  { key: "2", label: "Tue" },
  { key: "3", label: "Wed" },
  { key: "4", label: "Thu" },
  { key: "5", label: "Fri" },
  { key: "6", label: "Sat" },
  { key: "0", label: "Sun" },
];

const STATUS_SEGMENTS: Array<{ key: string; label: string; color: string }> = [
  { key: "completed", label: "Completed", color: "#5b8def" },
  { key: "playing", label: "Playing", color: "#64e67d" },
  { key: "paused", label: "Paused", color: "#e6c84f" },
  { key: "planned", label: "Planned", color: "#8a8f99" },
  { key: "dropped", label: "Dropped", color: "#e06a5c" },
];

export function renderStatisticsView(root: HTMLElement): void {
  clear(root);
  const body = el("div", { class: "view-body" });

  // `showLoading` only on the first open. On a granularity change we keep the
  // current content on screen and swap it atomically once the new data arrives,
  // so the page never collapses to an empty "Loading…" state (which looked like
  // a full reload: the layout shrank, scroll jumped to top, then re-expanded).
  const load = async (showLoading: boolean): Promise<void> => {
    if (showLoading) body.replaceChildren(el("p", { class: "muted" }, ["Loading…"]));
    try {
      const stats = await statisticsApi.get(granularity);
      body.replaceChildren(renderStats(stats));
    } catch (e) {
      body.replaceChildren(el("p", { class: "form-error" }, [`Failed to load statistics: ${String(e)}`]));
    }
  };

  root.append(buildHeader(() => void load(false)), body);
  void load(true);
}

function buildHeader(reload: () => void): HTMLElement {
  const buttons = new Map<StatsGranularity, HTMLButtonElement>();
  const seg = el(
    "div",
    { class: "seg" },
    GRANULARITIES.map((option) => {
      const button = el(
        "button",
        {
          type: "button",
          class: `seg-btn${granularity === option.id ? " on" : ""}`,
          onclick: () => {
            granularity = option.id;
            for (const [id, btn] of buttons) btn.classList.toggle("on", id === granularity);
            reload();
          },
        },
        [option.label],
      );
      buttons.set(option.id, button);
      return button;
    }),
  );

  return el("div", { class: "view-header" }, [
    el("h1", {}, ["Statistics"]),
    el("div", { class: "stats-gran" }, [el("span", { class: "muted sort-label" }, ["Over time"]), seg]),
  ]);
}

function renderStats(stats: Statistics): HTMLElement {
  return el("div", { class: "stats" }, [
    summarySection(stats),
    heatmapSection(stats.daily),
    el("div", { class: "grid2" }, [overTimeSection(stats.overTime), weekdaySection(stats.weekday)]),
    el("div", { class: "grid2" }, [
      chartCard("Top games", "by play time", horizontalBars(stats.topGames, "blue", compactDuration)),
      chartCard("Time by genre", "hours", horizontalBars(stats.genres, "green", compactDuration)),
    ]),
    el("div", { class: "grid2" }, [statusSection(stats.statuses), ratingsSection(stats.ratings)]),
  ]);
}

// --- KPI row ---------------------------------------------------------------

function summarySection(stats: Statistics): HTMLElement {
  const s = stats.summary;
  return el("section", { class: "kpis" }, [
    kpiTile("Total play time", formatTrackedDuration(s.totalPlaySeconds)),
    kpiTile("This year", formatTrackedDuration(s.yearPlaySeconds), true),
    kpiTile("Completed", String(s.completedCount)),
    kpiTile("Playing now", String(s.playingCount), s.playingCount > 0),
    kpiTile("Backlog", String(s.backlogCount)),
    kpiTile("Avg rating", s.averageRating === null ? "—" : `${s.averageRating.toFixed(1)}/10`),
  ]);
}

function kpiTile(label: string, value: string, green = false): HTMLElement {
  return el("div", { class: "kpi-tile" }, [
    el("span", { class: "kpi-label" }, [label]),
    el("b", { class: `kpi-value${green ? " green" : ""}` }, [value]),
  ]);
}

// --- Heatmap ---------------------------------------------------------------

function heatmapSection(daily: StatBar[]): HTMLElement {
  const seconds = new Map(daily.map((bar) => [bar.key, bar.value]));
  const today = new Date();
  const start = startOfLocalWeek(today);
  start.setDate(start.getDate() - (HEATMAP_WEEKS - 1) * 7);

  const cells: HTMLElement[] = [];
  const monthCells: HTMLElement[] = [];
  let previousMonth = -1;

  for (let week = 0; week < HEATMAP_WEEKS; week += 1) {
    const columnDate = new Date(start.getTime() + week * 7 * DAY_MS);
    const month = columnDate.getMonth();
    const showLabel = month !== previousMonth;
    previousMonth = month;
    monthCells.push(
      el("span", {}, [showLabel ? columnDate.toLocaleDateString(undefined, { month: "short" }) : ""]),
    );

    for (let day = 0; day < 7; day += 1) {
      const date = new Date(start.getTime() + (week * 7 + day) * DAY_MS);
      if (date.getTime() > today.getTime()) {
        cells.push(el("i", { class: "future" }));
        continue;
      }
      const value = seconds.get(localDateKey(date)) ?? 0;
      const level = heatLevel(value);
      const title = `${date.toLocaleDateString(undefined, { dateStyle: "medium" })} · ${
        value > 0 ? formatTrackedDuration(value) : "no play"
      }`;
      cells.push(el("i", { class: level > 0 ? `l${level}` : "", title }));
    }
  }

  const gridColumns = `repeat(${HEATMAP_WEEKS}, 12px)`;
  return el("section", { class: "card" }, [
    cardHead("Play activity", "all time · daily"),
    el("div", { class: "heat-scroll" }, [
      el("div", { class: "heat-months", style: `grid-template-columns:${gridColumns}` }, monthCells),
      el("div", { class: "heat", style: `grid-template-columns:${gridColumns}` }, cells),
    ]),
    el("div", { class: "heat-legend" }, [
      el("span", {}, ["Less"]),
      ...[0, 1, 2, 3, 4].map((level) => el("i", { class: level > 0 ? `l${level}` : "" })),
      el("span", {}, ["More"]),
    ]),
  ]);
}

function heatLevel(seconds: number): number {
  const minutes = seconds / 60;
  if (minutes <= 0) return 0;
  if (minutes < 30) return 1;
  if (minutes < 90) return 2;
  if (minutes < 180) return 3;
  return 4;
}

// --- Over-time series ------------------------------------------------------

function overTimeSection(overTime: StatBar[]): HTMLElement {
  const filled = fillTimeBuckets(overTime, granularity);
  const chart =
    filled.length === 0
      ? emptyChart("No play time tracked yet.")
      : verticalBars(
          filled.map((bucket) => ({
            value: bucket.value,
            caption: bucketLabel(bucket.key, granularity),
            title: `${bucketTitle(bucket.key, granularity)} · ${formatTrackedDuration(bucket.value)}`,
          })),
          true,
        );
  return chartCard("Play time over time", `by ${granularity}`, chart);
}

/** Fill missing buckets between the first and last present key with zero. */
function fillTimeBuckets(bars: StatBar[], gran: StatsGranularity): StatBar[] {
  if (bars.length === 0) return [];
  const value = new Map(bars.map((bar) => [bar.key, bar.value]));
  const first = bars[0]!.key;
  const last = bars[bars.length - 1]!.key;
  const keys: string[] = [];

  if (gran === "year") {
    for (let y = Number(first); y <= Number(last); y += 1) keys.push(String(y));
  } else if (gran === "month") {
    let y = Number(first.slice(0, 4));
    let m = Number(first.slice(5, 7));
    const endY = Number(last.slice(0, 4));
    const endM = Number(last.slice(5, 7));
    while (y < endY || (y === endY && m <= endM)) {
      keys.push(`${y}-${String(m).padStart(2, "0")}`);
      m += 1;
      if (m > 12) {
        m = 1;
        y += 1;
      }
    }
  } else {
    const end = parseLocalDate(last).getTime();
    for (let t = parseLocalDate(first).getTime(); t <= end; t += 7 * DAY_MS) {
      keys.push(localDateKey(new Date(t)));
    }
  }

  return keys.map((key) => ({ key, value: value.get(key) ?? 0 }));
}

function bucketLabel(key: string, gran: StatsGranularity): string {
  if (gran === "year") return key;
  if (gran === "month") {
    const date = new Date(Number(key.slice(0, 4)), Number(key.slice(5, 7)) - 1, 1);
    const month = date.toLocaleDateString(undefined, { month: "short" });
    return date.getMonth() === 0 ? `${month} ’${key.slice(2, 4)}` : month;
  }
  return parseLocalDate(key).toLocaleDateString(undefined, { day: "numeric", month: "short" });
}

function bucketTitle(key: string, gran: StatsGranularity): string {
  if (gran === "year") return key;
  if (gran === "month") {
    return new Date(Number(key.slice(0, 4)), Number(key.slice(5, 7)) - 1, 1).toLocaleDateString(
      undefined,
      { month: "long", year: "numeric" },
    );
  }
  return `Week of ${parseLocalDate(key).toLocaleDateString(undefined, { dateStyle: "medium" })}`;
}

// --- Weekday ---------------------------------------------------------------

function weekdaySection(weekday: StatBar[]): HTMLElement {
  const seconds = new Map(weekday.map((bar) => [bar.key, bar.value]));
  let peak = -1;
  let peakValue = -1;
  WEEKDAYS.forEach((day, index) => {
    const value = seconds.get(day.key) ?? 0;
    if (value > peakValue) {
      peakValue = value;
      peak = index;
    }
  });

  const bars = WEEKDAYS.map((day, index) => {
    const value = seconds.get(day.key) ?? 0;
    return {
      value,
      caption: day.label,
      highlight: peakValue > 0 && index === peak,
      title: `${day.label} · ${value > 0 ? formatTrackedDuration(value) : "no play"}`,
    };
  });

  const chart = peakValue > 0 ? verticalBars(bars) : emptyChart("No play time tracked yet.");
  return chartCard("When you play", "by weekday", chart);
}

// --- Status donut ----------------------------------------------------------

function statusSection(statuses: StatBar[]): HTMLElement {
  const counts = new Map(statuses.map((bar) => [bar.key, bar.value]));
  const total = statuses.reduce((sum, bar) => sum + bar.value, 0);

  let body: HTMLElement;
  if (total === 0) {
    body = emptyChart("No games yet.");
  } else {
    const stops: string[] = [];
    let cursor = 0;
    for (const segment of STATUS_SEGMENTS) {
      const count = counts.get(segment.key) ?? 0;
      if (count === 0) continue;
      const end = cursor + (count / total) * 100;
      stops.push(`${segment.color} ${cursor}% ${end}%`);
      cursor = end;
    }
    const donut = el("div", { class: "donut", style: `background:conic-gradient(${stops.join(",")})` }, [
      el("div", { class: "donut-hole" }, [
        el("b", {}, [String(total)]),
        el("span", {}, ["games"]),
      ]),
    ]);
    const legend = el(
      "ul",
      { class: "donut-legend" },
      STATUS_SEGMENTS.map((segment) =>
        el("li", {}, [
          el("i", { style: `background:${segment.color}` }),
          el("span", {}, [segment.label]),
          el("b", {}, [String(counts.get(segment.key) ?? 0)]),
        ]),
      ),
    );
    body = el("div", { class: "donut-wrap" }, [donut, legend]);
  }

  return el("section", { class: "card" }, [cardHead("Library by status", ""), body]);
}

// --- Ratings ---------------------------------------------------------------

function ratingsSection(ratings: StatBar[]): HTMLElement {
  const counts = new Map(ratings.map((bar) => [bar.key, bar.value]));
  const total = ratings.reduce((sum, bar) => sum + bar.value, 0);
  const bars = Array.from({ length: 10 }, (_, index) => {
    const score = index + 1;
    const value = counts.get(String(score)) ?? 0;
    return {
      value,
      caption: String(score),
      title: `${score}/10 · ${value} ${value === 1 ? "game" : "games"}`,
    };
  });
  const chart = total > 0 ? verticalBars(bars, false, "ratings") : emptyChart("No rated games yet.");
  return chartCard("Ratings", "games rated", chart);
}

// --- Reusable chart primitives ---------------------------------------------

interface VBar {
  value: number;
  caption: string;
  title?: string;
  highlight?: boolean;
}

function verticalBars(data: VBar[], scroll = false, extraClass = ""): HTMLElement {
  const max = Math.max(1, ...data.map((bar) => bar.value));
  const columns = data.map((bar) => {
    const heightPct = bar.value > 0 ? Math.max(2, (bar.value / max) * 100) : 0;
    const barEl = el("div", { class: "vbar", style: `height:${heightPct}%`, title: bar.title ?? "" });
    return el("div", { class: `vcol${bar.highlight ? " hl" : ""}` }, [
      el("div", { class: "vbar-track" }, [barEl]),
      el("span", { class: "vcap" }, [bar.caption]),
    ]);
  });
  const chart = el("div", { class: `vbars ${extraClass}` }, columns);
  return scroll ? el("div", { class: "vbars-scroll" }, [chart]) : chart;
}

function horizontalBars(
  data: StatBar[],
  fill: "blue" | "green",
  formatValue: (value: number) => string,
): HTMLElement {
  if (data.length === 0) return emptyChart("No data yet.");
  const max = Math.max(1, ...data.map((bar) => bar.value));
  return el(
    "div",
    { class: "hbars" },
    data.map((bar) =>
      el("div", { class: "hbar-row" }, [
        el("span", { class: "hbar-name", title: bar.key }, [bar.key]),
        el("div", { class: "hbar-track" }, [
          el("div", { class: `hbar-fill ${fill}`, style: `width:${(bar.value / max) * 100}%` }),
        ]),
        el("span", { class: "hbar-val" }, [formatValue(bar.value)]),
      ]),
    ),
  );
}

// --- Small helpers ---------------------------------------------------------

function chartCard(title: string, subtitle: string, chart: HTMLElement): HTMLElement {
  return el("section", { class: "card" }, [cardHead(title, subtitle), chart]);
}

function cardHead(title: string, subtitle: string): HTMLElement {
  return el("div", { class: "card-head" }, [
    el("h3", {}, [title]),
    ...(subtitle ? [el("span", { class: "card-sub" }, [subtitle])] : []),
  ]);
}

function emptyChart(message: string): HTMLElement {
  return el("p", { class: "muted chart-empty" }, [message]);
}

function compactDuration(seconds: number): string {
  if (seconds >= 3600) return `${Math.round(seconds / 3600)}h`;
  if (seconds >= 60) return `${Math.round(seconds / 60)}m`;
  return "0";
}

function localDateKey(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

function parseLocalDate(key: string): Date {
  return new Date(`${key}T00:00:00`);
}

function pad(value: number): string {
  return String(value).padStart(2, "0");
}
