import { gamesApi } from "../api/games";
import { clear, el } from "../ui/dom";
import {
  formatCalendarPlayPeriod,
  formatRelativeTime,
  formatSessionTimer,
  formatTrackedDuration,
} from "../ui/format";
import {
  GAME_SORTS,
  VISIBLE_GAME_STATUSES,
  SORT_LABELS,
  STATUS_LABELS,
  type Game,
  type GameQuery,
  type GameSort,
  type GameStatus,
  type GenreMatch,
  type SortDirection,
} from "../types/game";
import { openGameForm } from "./game-form";
import { renderGameDetail } from "./game-detail";

type YearKind = "" | "release" | "started" | "finished" | "played";

interface BrowserState {
  search: string;
  statuses: GameStatus[];
  genresText: string;
  genreMatch: GenreMatch;
  platform: string;
  minRating: string;
  maxRating: string;
  yearKind: YearKind;
  year: string;
  sort: GameSort;
  direction: SortDirection;
  advancedOpen: boolean;
}

// Persisted across navigation so returning from a game restores the browser.
let state: BrowserState = {
  search: "",
  statuses: [],
  genresText: "",
  genreMatch: "any",
  platform: "",
  minRating: "",
  maxRating: "",
  yearKind: "",
  year: "",
  sort: "default",
  direction: "desc",
  advancedOpen: false,
};

function parseList(value: string): string[] {
  return value
    .split(",")
    .map((v) => v.trim())
    .filter((v) => v.length > 0);
}

function toInt(value: string): number | undefined {
  const v = value.trim();
  if (v === "") return undefined;
  const n = Number(v);
  return Number.isFinite(n) ? Math.trunc(n) : undefined;
}

/** Translate the UI filter state into the backend query, omitting empty filters. */
function buildQuery(): GameQuery {
  const q: GameQuery = { sort: state.sort, direction: state.direction };

  if (state.search.trim()) q.search = state.search.trim();
  if (state.statuses.length) q.statuses = [...state.statuses];

  const genres = parseList(state.genresText);
  if (genres.length) {
    q.genres = genres;
    q.genreMatch = state.genreMatch;
  }

  const platforms = parseList(state.platform);
  if (platforms.length) q.platforms = platforms;

  const min = toInt(state.minRating);
  if (min !== undefined) q.minRating = min;
  const max = toInt(state.maxRating);
  if (max !== undefined) q.maxRating = max;

  const year = toInt(state.year);
  if (year !== undefined && state.yearKind !== "") {
    if (state.yearKind === "release") q.releaseYear = year;
    else if (state.yearKind === "started") q.startedYear = year;
    else if (state.yearKind === "finished") q.finishedYear = year;
    else if (state.yearKind === "played") q.playedYear = year;
  }

  return q;
}

export function renderGamesView(root: HTMLElement): void {
  const results = el("div", { class: "view-body" });
  const reload = async (): Promise<void> => {
    clear(results);
    results.append(el("p", { class: "muted" }, ["Loading…"]));
    try {
      const games = await gamesApi.list(buildQuery());
      clear(results);
      results.append(games.length === 0 ? emptyState() : gamesTable(games, root));
    } catch (e) {
      clear(results);
      results.append(el("p", { class: "form-error" }, [`Failed to load games: ${String(e)}`]));
    }
  };

  clear(root);
  root.append(buildHeader(reload), buildToolbar(reload), results);
  void reload();
}

function buildHeader(reload: () => Promise<void>): HTMLElement {
  return el("div", { class: "view-header" }, [
    el("h1", {}, ["Games"]),
    el(
      "button",
      {
        class: "btn btn-primary",
        onclick: () =>
          openGameForm({
            game: null,
            onSubmit: async (i) => void (await gamesApi.create(i), await reload()),
          }),
      },
      ["+ Add game"],
    ),
  ]);
}

function buildToolbar(reload: () => Promise<void>): HTMLElement {
  // Search (debounced).
  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  const search = el("input", {
    class: "search",
    type: "search",
    placeholder: "Search by title…",
    value: state.search,
    oninput: (e: Event) => {
      state.search = (e.target as HTMLInputElement).value;
      if (searchTimer) clearTimeout(searchTimer);
      searchTimer = setTimeout(() => void reload(), 200);
    },
  });

  // Sort field + direction toggle.
  const sortSelect = el(
    "select",
    {
      onchange: (e: Event) => {
        state.sort = (e.target as HTMLSelectElement).value as GameSort;
        // The default ordering is fixed, so direction does not apply to it.
        directionBtn.classList.toggle("hidden", state.sort === "default");
        void reload();
      },
    },
    GAME_SORTS.map((s) => el("option", { value: s, selected: state.sort === s }, [SORT_LABELS[s]])),
  );
  const directionBtn = el(
    "button",
    {
      class: `btn${state.sort === "default" ? " hidden" : ""}`,
      title: "Toggle sort direction",
      onclick: () => {
        state.direction = state.direction === "asc" ? "desc" : "asc";
        directionBtn.textContent = state.direction === "asc" ? "↑ Asc" : "↓ Desc";
        void reload();
      },
    },
    [state.direction === "asc" ? "↑ Asc" : "↓ Desc"],
  );

  const advanced = buildAdvancedPanel(reload);
  const filtersBtn = el(
    "button",
    {
      class: "btn",
      onclick: () => {
        state.advancedOpen = !state.advancedOpen;
        advanced.classList.toggle("hidden", !state.advancedOpen);
      },
    },
    ["Filters"],
  );

  const topRow = el("div", { class: "toolbar-row" }, [
    search,
    el("div", { class: "toolbar-spacer" }),
    filtersBtn,
    el("span", { class: "muted sort-label" }, ["Sort"]),
    sortSelect,
    directionBtn,
  ]);

  return el("div", { class: "toolbar" }, [topRow, buildStatusChips(reload), advanced]);
}

function buildStatusChips(reload: () => Promise<void>): HTMLElement {
  const chips = VISIBLE_GAME_STATUSES.map((s) => {
    const chip = el(
      "button",
      {
        class: `chip${state.statuses.includes(s) ? " active" : ""}`,
        onclick: () => {
          state.statuses = state.statuses.includes(s)
            ? state.statuses.filter((x) => x !== s)
            : [...state.statuses, s];
          chip.classList.toggle("active", state.statuses.includes(s));
          void reload();
        },
      },
      [STATUS_LABELS[s]],
    );
    return chip;
  });
  return el("div", { class: "chips" }, chips);
}

function buildAdvancedPanel(reload: () => Promise<void>): HTMLElement {
  const genres = el("input", {
    type: "text",
    placeholder: "Action, RPG…",
    value: state.genresText,
    oninput: (e: Event) => {
      state.genresText = (e.target as HTMLInputElement).value;
    },
    onchange: () => void reload(),
  });
  const genreMatch = el(
    "select",
    {
      onchange: (e: Event) => {
        state.genreMatch = (e.target as HTMLSelectElement).value as GenreMatch;
        void reload();
      },
    },
    [
      el("option", { value: "any", selected: state.genreMatch === "any" }, ["Any"]),
      el("option", { value: "all", selected: state.genreMatch === "all" }, ["All"]),
    ],
  );

  const platform = el("input", {
    type: "text",
    placeholder: "PC, PS5…",
    value: state.platform,
    oninput: (e: Event) => {
      state.platform = (e.target as HTMLInputElement).value;
    },
    onchange: () => void reload(),
  });

  const minRating = numberInput("min", state.minRating, (v) => (state.minRating = v), reload);
  const maxRating = numberInput("max", state.maxRating, (v) => (state.maxRating = v), reload);

  const yearKind = el(
    "select",
    {
      onchange: (e: Event) => {
        state.yearKind = (e.target as HTMLSelectElement).value as YearKind;
        void reload();
      },
    },
    [
      el("option", { value: "", selected: state.yearKind === "" }, ["Year: off"]),
      el("option", { value: "release", selected: state.yearKind === "release" }, ["Released in"]),
      el("option", { value: "started", selected: state.yearKind === "started" }, ["Started in"]),
      el("option", { value: "finished", selected: state.yearKind === "finished" }, ["Finished in"]),
      el("option", { value: "played", selected: state.yearKind === "played" }, ["Played in"]),
    ],
  );
  const year = el("input", {
    type: "number",
    placeholder: "2024",
    value: state.year,
    oninput: (e: Event) => {
      state.year = (e.target as HTMLInputElement).value;
    },
    onchange: () => void reload(),
  });

  const group = (label: string, ...controls: HTMLElement[]): HTMLElement =>
    el("div", { class: "filter-group" }, [el("span", { class: "muted" }, [label]), ...controls]);

  return el("div", { class: `advanced${state.advancedOpen ? "" : " hidden"}` }, [
    group("Genres", genres, genreMatch),
    group("Platform", platform),
    group("Rating", minRating, el("span", { class: "muted" }, ["–"]), maxRating),
    group("Year", yearKind, year),
  ]);
}

function numberInput(
  placeholder: string,
  value: string,
  set: (v: string) => void,
  reload: () => Promise<void>,
): HTMLInputElement {
  return el("input", {
    class: "num-input",
    type: "number",
    placeholder,
    value,
    oninput: (e: Event) => set((e.target as HTMLInputElement).value),
    onchange: () => void reload(),
  });
}

function emptyState(): HTMLElement {
  return el("div", { class: "empty" }, [
    el("p", {}, ["No games match."]),
    el("p", { class: "muted" }, ["Adjust the filters, or add a game."]),
  ]);
}

function liveTimer(startedAt: string): HTMLElement {
  const timer = el("span", { class: "session-timer" }, [formatSessionTimer(startedAt)]);
  const interval = window.setInterval(() => {
    if (!timer.isConnected) window.clearInterval(interval);
    else timer.textContent = formatSessionTimer(startedAt);
  }, 1000);
  return timer;
}

function lastPlayedLabel(value: string, prefix = "Last played "): HTMLElement {
  const label = el("span", { class: "last-played" }, [`${prefix}${formatRelativeTime(value)}`]);
  const interval = window.setInterval(() => {
    if (!label.isConnected) window.clearInterval(interval);
    else label.textContent = `${prefix}${formatRelativeTime(value)}`;
  }, 60_000);
  return label;
}

function trackingCell(game: Game): HTMLElement {
  const content: Node[] = [];
  if (game.completedSessionsCount > 0) {
    content.push(el("strong", {}, [formatTrackedDuration(game.totalPlayTimeSeconds)]));
  } else if (!game.isPlayingNow) {
    content.push(el("strong", {}, ["—"]));
  }
  if (game.isPlayingNow && game.activeSessionStartedAt) {
    content.push(liveTimer(game.activeSessionStartedAt));
  } else if (game.status === "playing" && game.lastPlayedAt) {
    content.push(lastPlayedLabel(game.lastPlayedAt));
  }
  return el("div", { class: "tracking-cell" }, content);
}

function gamesTable(games: Game[], root: HTMLElement): HTMLElement {
  const rows = games.map((game) =>
    el("tr", {}, [
      el("td", {}, [
        el(
          "button",
          { class: "link", onclick: () => renderGameDetail(root, game.id, () => renderGamesView(root)) },
          [game.title],
        ),
        ...(game.isPlayingNow ? [el("span", { class: "playing-indicator", title: "Playing now" })] : []),
      ]),
      el("td", { class: "genres-cell" }, [game.genres.length ? game.genres.join(", ") : "—"]),
      el("td", { class: "period-cell" }, [formatCalendarPlayPeriod(game.startedAt, game.finishedAt)]),
      el("td", {}, [trackingCell(game)]),
      el("td", {}, [el("span", { class: `badge status-${game.status}` }, [STATUS_LABELS[game.status]])]),
      el("td", { class: "num" }, [game.rating === null ? "—" : `${game.rating}/10`]),
    ]),
  );

  return el("table", { class: "data-table" }, [
    el("thead", {}, [
      el("tr", {}, [
        el("th", {}, ["Title"]),
        el("th", {}, ["Genres"]),
        el("th", {}, ["Play period"]),
        el("th", {}, ["Play time"]),
        el("th", {}, ["Status"]),
        el("th", { class: "num" }, ["Rating"]),
      ]),
    ]),
    el("tbody", {}, rows),
  ]);
}
