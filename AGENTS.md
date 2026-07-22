# AGENTS.md

> **Single source of truth for AI agents working on Xareon.**
> After **every** change to the project, update this file so it always reflects the
> current implementation. Keep it accurate over comprehensive.

---

## 1. Purpose

**Xareon** is a *personal desktop gaming journal* — not a games catalog. It exists to
record one player's memories, thoughts, achievements, screenshots and personal history
across many years. It is intentionally **not** about cataloging every game in the world;
it captures *your* experience.

## 2. Architecture overview

Xareon is a [Tauri v2](https://tauri.app/) desktop app. **All application logic lives in
the Rust backend**; the TypeScript frontend is a thin, framework-free UI that talks to the
backend exclusively through Tauri commands.

The Rust side follows Clean Architecture. Dependencies point inward only:

```
commands  →  services  →  repositories  →  db
                   ↘         ↘
                    domain (shared models)
                    error  (shared error type)
```

- **commands** — Tauri command handlers. The boundary to the frontend. Wire up a
  repository + service per call; no business logic.
- **services** — business rules and validation. Orchestrate repositories. No SQL, no UI.
- **repositories** — the **only** layer that contains SQL. Defined as traits with
  SQLite implementations so the storage engine can change without touching services.
- **db** — the `DatabaseManager` gateway (sole owner of the connection), opening the
  connection and running versioned migrations.
- **domain** — plain data models (`Game`, `GameStatus`, `GameInput`). No persistence or
  UI knowledge.
- **error** — single `AppError` type that flows to the command boundary and serializes to
  a message string for the frontend.

Supporting layers (used by services as they grow):

- **validation** — reusable validation rules (`require_non_empty`, `require_in_range`).
- **storage** — file-backed features. `profile_sync` owns SQLite snapshots, backup
  manifests, checksums, safety copies and per-device sync state.
- **config** — application configuration (reserved).
- **events** — domain events (reserved).

The frontend mirrors this separation:

```
main.ts (shell/nav)  →  views/*  →  api/*  →  Tauri invoke
                                     types/*  (mirror of domain models)
```

- **api/** — thin typed wrappers around `invoke`. The UI never calls `invoke` directly.
- **views/** — render DOM and handle interaction. No `invoke`, no business logic.
- **types/** — TypeScript mirrors of the Rust domain types (camelCase wire format).

## 3. Directory structure

```
xareon/
├── index.html                  # Vite entry
├── package.json                # frontend deps + scripts
├── public/
│   └── xareon-icon.png         # frontend copy of the app icon, used in the sidebar brand
├── tsconfig.json               # strict TypeScript config
├── vite.config.ts              # Vite/Tauri dev server config
├── AGENTS.md                   # this file
├── idea.md                     # original product specification
├── ideas/db-sync.md            # Ukrainian design for manual profile synchronization
├── ideas/auto-tracking.md      # Ukrainian design for Windows-only automatic play tracking
├── src/                        # frontend (TypeScript, UI only)
│   ├── main.ts                 # app shell + sidebar navigation
│   ├── styles.css              # dark, minimal theme
│   ├── api/
│   │   ├── achievements.ts     # wrappers over achievement commands
│   │   ├── automatic-tracking.ts # Windows process bindings + tracking status
│   │   ├── games.ts            # typed wrappers over game_* + list_genres
│   │   ├── journal.ts          # wrappers over *_journal_entry commands
│   │   ├── play-sessions.ts    # manual play tracking + today/week play-time totals
│   │   ├── statistics.ts       # wrapper over get_statistics
│   │   └── settings.ts         # settings + profile backup command wrappers
│   ├── types/
│   │   ├── achievement.ts      # Achievement/AchievementStatus + input types
│   │   ├── automatic-tracking.ts # executable/process/status wire models
│   │   ├── game.ts             # Game/GameInput/GameStatus + GameQuery/sort types
│   │   ├── genre.ts            # Genre
│   │   ├── journal.ts          # JournalEntry + inputs
│   │   ├── play-session.ts     # active PlaySession + DailyPlayTime/PlayTimeTotals
│   │   ├── statistics.ts       # Statistics aggregate + StatBar/StatsSummary/granularity
│   │   └── settings.ts         # Settings + ProfileSyncInfo/status
│   ├── ui/
│   │   ├── dom.ts              # tiny typed DOM helpers (el, clear)
│   │   └── format.ts           # date/time formatting
│   └── views/
│       ├── games-view.ts       # game browser: filters, sort, table + today/week play summary
│       ├── game-form.ts        # create/edit modal form (multi-genre input)
│       ├── game-detail.ts      # tabbed game detail (overview/achievements/journal/details) + Edit/Delete
│       ├── statistics-view.ts  # all-time dashboard: KPIs, heatmap, hand-rolled charts
│       └── settings-view.ts    # settings + manual Google Drive folder backup/restore
└── src-tauri/                  # backend (Rust)
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json         # window, bundle, dev/build commands
    ├── capabilities/
    │   └── default.json        # permissions for the main window
    ├── icons/                  # generated app icons (see §9)
    └── src/
        ├── main.rs             # thin entry point → xareon_lib::run()
        ├── lib.rs              # builds the Tauri app, registers state + commands
        ├── state.rs            # AppState { db: DatabaseManager }
        ├── error.rs            # AppError / AppResult
        ├── domain/
        │   ├── achievement.rs  # Achievement, AchievementStatus + inputs
        │   ├── automatic_tracking.rs # bindings, processes, states + capabilities
        │   ├── game.rs         # Game, GameInput, GameStatus, GameQuery + sort/filter enums
        │   ├── genre.rs        # Genre + normalize()
        │   ├── journal.rs      # JournalEntry, NewJournalEntry, JournalEntryUpdate
        │   ├── play_session.rs # active tracking state + daily play-time aggregate
        │   ├── statistics.rs   # Statistics aggregate + StatBar/StatsSummary/StatsGranularity
        │   └── settings.rs     # Settings (typed aggregate of app settings)
        ├── repositories/
        │   ├── achievement_repository.rs # achievements table
        │   ├── game_process_repository.rs # executable bindings + enable flag
        │   ├── game_repository.rs     # games table + browser query + genre hydration
        │   ├── genre_repository.rs    # genres + game_genres writes (get_or_create, links)
        │   ├── journal_repository.rs  # journal_entries
        │   ├── play_session_repository.rs # active tracking + daily aggregates + cached totals
        │   ├── statistics_repository.rs # read-only GROUP BY aggregates for the dashboard
        │   └── settings_repository.rs # settings key-value store (get_all/set)
        ├── services/
        │   ├── achievement_service.rs # validation + progress/status rules
        │   ├── automatic_tracking_service.rs # binding rules/settings
        │   ├── game_service.rs     # GameService (validation + game/genre orchestration)
        │   ├── genre_service.rs    # GenreService (list genres)
        │   ├── journal_service.rs  # JournalService (validation, ensures game exists)
        │   ├── play_session_service.rs # single-session tracking lifecycle
        │   ├── statistics_service.rs # assembles the Statistics payload
        │   └── settings_service.rs # SettingsService (maps typed Settings ↔ KV keys)
        ├── validation/         # reusable business validation rules
        │   └── mod.rs          # require_non_empty, require_in_range
        ├── storage/
        │   └── profile_sync.rs # SQLite snapshot/restore, manifest, hash + local sync state
        ├── config/
        │   ├── mod.rs
        │   ├── automatic_tracking.rs # Windows poll worker, AFK + suppression
        │   ├── device_settings.rs # local shortcut + registration error, never synchronized
        │   ├── global_shortcut.rs # macOS/Windows shortcut registration adapter
        │   └── session_indicator.rs # active-session menu bar/system tray lifecycle
        ├── events/            # domain events — reserved for future use
        ├── commands/
        │   ├── achievement_commands.rs # achievement #[tauri::command] handlers
        │   ├── automatic_tracking_commands.rs # processes, bindings + status
        │   ├── game_commands.rs     # game #[tauri::command] handlers (writes in a tx)
        │   ├── genre_commands.rs    # list_genres
        │   ├── journal_commands.rs  # journal #[tauri::command] handlers
        │   ├── play_session_commands.rs # Play/Stop/heartbeat + play-time totals + Dock icon
        │   ├── profile_sync_commands.rs # folder picker, backup/restore + reveal DB
        │   ├── statistics_commands.rs # get_statistics (read-only)
        │   └── settings_commands.rs # get_settings, update_settings (write in a tx)
        ├── db/
        │   ├── connection.rs   # open() + enable FKs + run migrations
        │   ├── manager.rs      # DatabaseManager — sole gateway to the connection
        │   └── migrations.rs   # versioned migration runner (user_version)
        └── migrations/
            ├── 0001_init.sql            # games table
            ├── 0002_genres_journal.sql  # genres, game_genres, journal_entries
            ├── 0003_settings.sql        # settings key-value store
            ├── 0004_achievements.sql    # user-defined personal achievements
            ├── 0005_play_sessions.sql   # play history + cached totals
            ├── 0006_endless_status.sql  # rebuild games table to add the 'endless' status
            ├── 0007_automatic_tracking.sql # bindings + legacy session source/reason
            └── 0008_daily_play_time.sql # active state + migrated daily aggregates
```

## 4. Technology stack

- **Tauri v2** — desktop shell.
- **Rust** — backend / all application logic.
- **rusqlite** (`bundled`, `backup`) — compiled-in SQLite plus Online Backup API snapshots.
- **serde / serde_json** — (de)serialization across the command boundary.
- **sha2** — SHA-256 integrity and freshness comparison for profile backups.
- **rfd** — native macOS/Windows directory picker for the per-device sync folder.
- **windows-sys** (Windows only) — process/window/last-input observation for automatic
  play tracking; Xareon installs no keyboard or mouse hooks.
- **thiserror** — error type derivation.
- **TypeScript** (strict) + **Vite** — frontend build. **No** UI framework (no React /
  Vue / Angular / Svelte). Vanilla DOM via small helpers.
- **@tauri-apps/api** — `invoke` bridge.

## 5. Build & run

Prerequisites: **Node.js** (with npm) and the **Rust toolchain** (`rustup`, stable).

```bash
npm install                 # install frontend deps (first time)
npm run tauri dev           # run the app in development (hot-reloads frontend)
npm run tauri build         # produce a production bundle
npm run build               # frontend type-check (tsc) + production build only
cd src-tauri && cargo build # compile the backend only
```

The database lives in the OS app-data directory (`app_data_dir()/xareon.db`) and is
created with migrations applied on first launch.

## 6. Database

- **Engine:** SQLite via `rusqlite` (bundled).
- **Foreign keys:** enabled per connection (`PRAGMA foreign_keys = ON`).
- **Migrations:** versioned via `PRAGMA user_version`. Each migration is a `.sql` file in
  `src-tauri/src/migrations/`, embedded at compile time and listed in order in
  `db/migrations.rs`. On open, every migration whose 1-based index exceeds the current
  `user_version` is applied and the version advanced.
- **No raw SQL outside repositories.**
- **Transactions:** writes that span tables (creating/updating a game plus its genre
  links) run inside a transaction opened at the command boundary
  (`Connection::unchecked_transaction`), so the aggregate commits atomically.

### Schema (current)

`games` — no `genre` column (genres are normalized; see below).
| column        | type    | notes                                                  |
|---------------|---------|--------------------------------------------------------|
| id            | INTEGER | PK, autoincrement                                      |
| title         | TEXT    | NOT NULL                                               |
| platform      | TEXT    | nullable                                               |
| developer     | TEXT    | nullable                                               |
| publisher     | TEXT    | nullable                                               |
| release_year  | INTEGER | nullable                                               |
| started_at    | TEXT    | nullable, ISO date                                     |
| finished_at   | TEXT    | nullable, ISO date                                     |
| status        | TEXT    | NOT NULL, default `planned`, CHECK in status set       |
| rating        | INTEGER | nullable, CHECK 0–10                                    |
| cover_path    | TEXT    | nullable                                               |
| created_at    | TEXT    | NOT NULL, default `datetime('now')`                    |
| updated_at    | TEXT    | NOT NULL, default `datetime('now')`; set on update     |
| total_play_time_seconds | INTEGER | NOT NULL; cached completed-session total       |
| is_playing_now | INTEGER | boolean 0/1; fast projection of the active session    |
| last_played_at | TEXT    | nullable; end time of the latest completed session    |
| automatic_tracking_enabled | INTEGER | boolean 0/1; default off                 |

`genres` — reusable genre entities (a game can have many; a genre many games).
| column          | type    | notes                                              |
|-----------------|---------|----------------------------------------------------|
| id              | INTEGER | PK, autoincrement                                  |
| name            | TEXT    | NOT NULL, first-seen casing                         |
| name_normalized | TEXT    | NOT NULL, UNIQUE — trimmed+lowercased; dedupe key  |

`game_genres` — many-to-many link.
| column   | type    | notes                                          |
|----------|---------|------------------------------------------------|
| game_id  | INTEGER | FK → games(id) ON DELETE CASCADE               |
| genre_id | INTEGER | FK → genres(id) ON DELETE CASCADE              |
|          |         | PRIMARY KEY (game_id, genre_id)                |

`journal_entries` — per-game journal (a first-class entity).
| column     | type    | notes                                            |
|------------|---------|--------------------------------------------------|
| id         | INTEGER | PK, autoincrement                                |
| game_id    | INTEGER | NOT NULL, FK → games(id) ON DELETE CASCADE       |
| body       | TEXT    | NOT NULL                                         |
| created_at | TEXT    | NOT NULL, default `datetime('now')`              |
| updated_at | TEXT    | NOT NULL, default `datetime('now')`; set on edit |

`settings` — application settings as a key-value store (schema stays stable as
settings are added; a new setting is a new key, not a new column).
| column     | type | notes                                              |
|------------|------|----------------------------------------------------|
| key        | TEXT | PK                                                 |
| value      | TEXT | NOT NULL (cleared fields stored as empty string)   |
| updated_at | TEXT | NOT NULL, default `datetime('now')`; set on update |

`achievements` — user-defined personal milestones for a game. These are deliberately
universal rather than template-driven: a row can represent clearing a location,
maxing out gear, finishing an ending, a self-imposed challenge, or any other
game-specific accomplishment.
| column           | type    | notes                                                      |
|------------------|---------|------------------------------------------------------------|
| id               | INTEGER | PK, autoincrement                                          |
| game_id          | INTEGER | NOT NULL, FK → games(id) ON DELETE CASCADE                 |
| title            | TEXT    | NOT NULL                                                   |
| description      | TEXT    | nullable                                                   |
| category         | TEXT    | nullable, free text (e.g. Locations, Gear, Endings)        |
| status           | TEXT    | NOT NULL, CHECK `planned`/`in_progress`/`completed`        |
| progress_current | INTEGER | nullable, CHECK >= 0                                       |
| progress_target  | INTEGER | nullable, CHECK > 0; current cannot exceed target          |
| progress_unit    | TEXT    | nullable (%, level, secrets, locations, etc.)              |
| completed_at     | TEXT    | nullable; set when completed, cleared when reopened        |
| is_hidden        | INTEGER | boolean 0/1 for non-obvious or spoiler-like achievements   |
| display_order    | INTEGER | NOT NULL, manual ordering within a game                    |
| created_at       | TEXT    | NOT NULL, default `datetime('now')`                        |
| updated_at       | TEXT    | NOT NULL, default `datetime('now')`; set on update         |

**Game statuses:** `planned`, `playing`, `paused`, `completed`, `completed_100`, `dropped`,
`endless`. `endless` marks evergreen games with no ending (MMOs, roguelikes, live-service,
sandboxes): they never reach `completed`, keep the "so far" open play period, and are
excluded from the Statistics completed/backlog KPIs while getting their own donut segment.
Whether a game is currently being played is read from the live session/last-played, not the
status — so `endless` needs no active/paused variants. Adding a status requires a table
rebuild migration (SQLite cannot ALTER a CHECK); see `0006_endless_status.sql`.
**Achievement statuses:** `planned`, `in_progress`, `completed`.

`active_play_session` — singleton runtime tracking state. Its `singleton_id` is always
`1`, so at most one game can be tracked globally. Start inserts it, the backend heartbeat
updates `last_activity_at`, and Stop atomically folds its exact elapsed interval into
`daily_play_time` before deleting it. Timestamps are UTC; `tracking_source` is `manual`
or `automatic`.

`daily_play_time` — compact permanent history with one row per game and local calendar
date (`PRIMARY KEY (game_id, play_date)`). It stores `duration_seconds`, split
`manual_seconds`/`automatic_seconds`, `sessions_count`, and the day's earliest/latest UTC
boundaries. Repeated short play periods update the same row. A period crossing local
midnight is split at every local-day boundary without losing elapsed seconds, including
across DST. Migration `0008` performs the same split for legacy `play_sessions`, preserves
an unfinished interval as `active_play_session`, then removes the legacy table.

`game_executable_bindings` — Windows executable paths used by automatic tracking. A game
can own several bindings; normalized case-insensitive paths are globally unique. These
profile rows are harmless on macOS, where the monitor and its UI are absent.

## 7. Modules (current)

- **Games** — full CRUD plus a flexible browser query. Commands: `list_games` (takes an
  optional `GameQuery`), `get_game`, `create_game`, `update_game`, `delete_game`.
  - The browser table has **no per-row Edit/Delete actions**. Editing and deleting a game
    live on the game detail page header (Edit opens the shared game form; Delete goes
    through `confirmDialog` and navigates back on success). The list is a pure read view.
  - The Games header shows a compact **today / this week** play-time summary next to the
    Add game button (see Play tracking below).
  - The browser omits platform as a dedicated column; platform remains available in the
    game form, detail view, and advanced browser filter. Its place is used by the
    separate Play period and Play time measures described below.
  - The browser and game overview keep calendar **Play period** separate from tracked
    **Play time**. Play period is a human calendar duration from `startedAt` to
    `finishedAt` (`1 month, 4 days`), or to today with `so far` while unfinished.
    `Game.playPeriodsCount` is the sum of daily `sessions_count` values and lets the
    UI distinguish missing historical tracking data from a real short period. Any game
    with no completed periods displays `—` regardless of status (including planned,
    paused and dropped legacy rows). A real completed period under one minute displays
    `<1m`; an active session remains visible through its separate live timer.
  - `completed_100` remains a valid backend/database status for compatibility but is
    intentionally absent from filters and game forms. Existing rows with that status are
    rendered everywhere as ordinary `Completed` and become `completed` if edited/saved.
  - **GameQuery** is the single query surface (not many narrow endpoints): `search`
    (title), `statuses` (OR), `platforms` (OR), `genres` + `genreMatch` (any/all),
    `releaseYear`/`startedYear`/`finishedYear`/`playedYear`, `minRating`/`maxRating`,
    plus `sort` (default/title/started/finished/release year/rating/status) and
    `direction` (asc/desc). New filters are added to `GameQuery` + `build_filters`,
    nowhere else.
  - **Default sort** (`GameSort::Default`, the sort when the page opens) is a fixed
    composite: currently-**playing** games first, then by **finished date descending**
    (NULLs last). It ignores `direction`; the user can still pick any other sort field
    and direction from the controls (the direction toggle is hidden for the default sort).
- **Genres** — normalized, reusable, recognized by normalized name. A game owns a list of
  genre names; the service resolves them to entities and links them. Command: `list_genres`
  (used for input suggestions; ready for future genre stats/management).
- **Journal** — per-game diary, a first-class entity. Commands: `list_journal_entries`
  (newest first), `create_journal_entry`, `update_journal_entry`, `delete_journal_entry`.
  In the UI, opening a game exposes Journal as a dedicated game-detail tab. Long journal
  entry bodies are collapsed by default with a Show all/Show less toggle.
- **Achievements** — per-game user-defined accomplishments/personal milestones, not
  platform-specific achievement imports and not a fixed template system. Commands:
  `list_achievements`, `create_achievement`, `update_achievement`,
  `set_achievement_progress`, `complete_achievement`, `reopen_achievement`,
  `delete_achievement`. The service validates flexible optional progress and
  auto-completes an achievement when `progressCurrent >= progressTarget`. In the UI,
  opening a game exposes Achievements as a dedicated game-detail tab with grid cards,
  circular progress indicators, and free-text category grouping. Long achievement
  descriptions are collapsed by default with a Show all/Show less toggle.

Game detail is a tabbed frontend view: Overview (summary cards + latest journal context),
Achievements, Journal, and Details. The global sidebar Achievements entry remains a
disabled placeholder for a future cross-game dashboard.

- **Settings** — profile settings use SQLite's extensible key-value store; settings that
  belong to one installation use app-config JSON and are excluded from profile restore.
  Commands: `get_settings`, `update_settings` (loads/saves the whole `Settings`
  aggregate). The typed `Settings` model maps to KV keys in `SettingsService` (the
  single mapping point); adding a setting is a field on `Settings` + a key there —
  **no migration**. `userIdentifier` is the human-readable public handle (not a UUID).
  Saving runs inside a transaction so all settings commit atomically. The profile sync
  folder is deliberately not a `Settings` field: it is device-specific and lives in
  app-config so restoring a Mac database cannot overwrite the Windows folder path.
  `playTrackingShortcut` is likewise device-local in `device-settings.json`, because
  macOS and Windows use different modifiers and reserve different combinations. The
  Settings UI captures a real key combination in a read-only shortcut input; Backspace
  disables it. Saving validates and replaces the OS registration. On startup an occupied
  or invalid shortcut is non-fatal: Xareon opens without it, persists the registration
  error locally and shows the warning on Settings so the user can choose another one.

- **Profile backup / manual synchronization** — uses a user-selected local folder that
  Google Drive for desktop synchronizes; there is no Google API or network code in
  Xareon. Commands: `choose_profile_sync_folder`, `get_profile_sync_info`,
  `upload_profile_backup`, `restore_profile_backup`, `open_database_folder`.
  `rfd` provides the native directory picker on macOS and Windows. Upload uses SQLite's
  Online Backup API and publishes `xareon-backup.sqlite` plus `xareon-backup.json`
  (SHA-256, size, schema version, UTC creation time and source platform) through temporary
  files. Restore validates the checksum, `PRAGMA integrity_check` and schema, stops an
  active play session, writes `app_data/backups/pre-restore-*.sqlite`, replaces the live
  database through the same Backup API and restarts Xareon. Per-device `profile-sync.json`
  stores the selected folder, operation dates and last common hash outside SQLite.
  Settings reports up-to-date/local-newer/backup-newer/conflict/unavailable/invalid states;
  conflict detection compares content hashes against that baseline, never file mtimes.

- **Play tracking** — manual and Windows-only automatic real-play periods. Commands:
  `get_active_play_session`, `list_game_daily_play_time`, `get_play_time_totals`,
  `get_game_play_time_today`, `start_play_session`,
  `heartbeat_play_session`, `stop_play_session`. Only one period
  can be active globally. Play/Stop atomically update the singleton active state, the
  relevant daily aggregates, and cached game fields. Startup closes an interrupted period at its last
  minute heartbeat; normal window close stops it at the current time. The browser and
  game detail show total time, Steam-like relative last-played time for games with status `playing`, a live
  `HH:MM:SS` timer and a subtle green indicator. Game detail has one Play/Stop control
  and hides Play while another game is active. Completed periods are retained as compact
  daily activity (`duration + period count`), not individual Start/Stop rows; game Overview
  lists recent days.
  - **Today / this-week play-time** is split between backend and frontend so reads stay
    cheap and the display stays live. `get_play_time_totals` returns a `PlayTimeTotals`
    (`todaySeconds`, `weekSeconds`) summed from `daily_play_time`; week starts Monday.
    `get_game_play_time_today` is the same for one game, today.
    These are simple indexed `SUM`s over a slowly-growing table — cheap, computed
    on-demand at view load, never on a timer. The frontend adds the **live** contribution
    of any active session (elapsed clamped to the local day/week start) on top of the
    backend snapshot and ticks it every second; on Stop a reload folds the finished
    period into the daily total, so there is no double counting. Helpers live in
    `ui/format.ts` (`activeSecondsToday`, `activeSecondsThisWeek`, `startOfLocalWeek`,
    `formatPlayTotal`). The Games header renders the global today/week pill; the game
    detail Overview grid shows a "Played today" card for that game. The Dock icon switches to a green
  Play-badged PNG during tracking. On macOS this uses native
  `NSApplication.setApplicationIconImage`; `window.set_icon` does not update the Dock.
  A configurable global shortcut uses the official Tauri global-shortcut plugin on
  macOS and Windows: it stops the active session or starts the most recently played game.
  The handler reuses repository lifecycle operations, updates the runtime icon and emits
  `play-tracking-changed` so an open UI refreshes and shows a short result toast.
  While a session is active, `session_indicator` shows a green Play icon in the macOS
  menu bar or Windows system tray. It is fully hidden without an active session. On
  macOS its title displays elapsed time without seconds (`2h 14m`); Windows does not
  support persistent tray title text, so the duration and game title are exposed through
  the tray tooltip. Clicking the indicator focuses the main Xareon window. A backend
  minute worker owns both heartbeat updates and indicator refreshes, so crash recovery
  remains accurate even when game detail is not open or the window is in the background.
  - On Windows, each game can bind several executable paths and opt into automatic
    tracking (off by default). The Details tab exposes this only through a backend platform
    capability. A backend worker observes processes, visible top-level windows,
    foreground/minimized state and `GetLastInputInfo`. It starts after foreground plus new
    input, stops on process exit, and ends after a 10-minute unfocused/idle grace period
    included in play time; returning from AFK creates a new period. Manual periods have
    priority, and manually stopping an automatic session suppresses restart until all
    bound processes exit. Automatic start/stop/AFK transitions emit
    `play-tracking-changed`; an open game detail reloads itself in place (preserving its
    selected tab), so the Play/Stop control, live timer, green indicator, automatic status
    and history change without a manual refresh. Recent per-game history shows one row per
    local day with total duration and play-period count.

- **Statistics** — an all-time dashboard over the tracked history. Command:
  `get_statistics(granularity)` → a single `Statistics` aggregate. Read-only; a handful of
  `GROUP BY … SUM/COUNT` queries run in one connection at view load (and on granularity
  change), never on a timer. Every play-time figure comes from `daily_play_time`; periods
  crossing midnight contribute to each correct local date.
  - Contents: KPI tiles (total play time, this year, completed, playing now, backlog,
    average rating); a GitHub-style daily **activity heatmap**; **play time over time**
    bucketed by `week`/`month`/`year` via the header granularity toggle (the only control —
    all other cards are all-time); **when you play** by weekday; **top games** and **time
    by genre** (a multi-genre game contributes to each of its genres); **library by status**
    (`completed_100` folded into `completed`); and a **ratings** histogram.
  - `StatBar { key, value }` is the shared point type — `value` is seconds for time series
    and a plain count otherwise; the frontend produces all display labels and fills missing
    time buckets. Charts are **hand-rolled** (framework-free, matching the UI convention):
    the heatmap and bars are CSS-sized `div`s, the status donut is a CSS `conic-gradient`.
    No charting library is used or wanted.

The frontend navigation lists Timeline and a global Achievements dashboard as disabled
placeholders. Per-game achievements are live inside game details; there is not yet a global
achievements dashboard. Games, Statistics and Settings are live nav entries.

## 8. Conventions

- **Rust:** modules grouped by layer; each layer has a `mod.rs` re-exporting submodules.
  `AppResult<T>` / `AppError` everywhere; `?` to propagate. SQL only in repositories.
  Domain enums bridge to SQLite via `ToSql`/`FromSql` impls in the repository layer (the
  domain stays persistence-agnostic).
- **Wire format:** structs crossing the boundary use `#[serde(rename_all = "camelCase")]`
  to match TypeScript. `GameStatus` serializes to its snake_case string (e.g.
  `completed_100`). The TS `types/game.ts` must stay in sync.
- **TypeScript:** strict mode with extra safety flags (`noUncheckedIndexedAccess`,
  `exactOptionalPropertyTypes`, `verbatimModuleSyntax`). UI calls `api/*`, never `invoke`
  directly. No business logic in views.
- **Dialogs:** never use the native `window.confirm/alert/prompt` — they are unreliable in
  the Tauri webview. Use `confirmDialog` (`src/ui/confirm.ts`). The Add/Edit game modal does
  **not** close on an outside click (only Cancel or a successful save), to avoid losing
  unsaved input.
- **Comments:** only where they add value. Consistent naming. No duplicated code.

## 9. Important architectural decisions

- **Logic in Rust, not TypeScript.** The frontend is a pure UI; all rules, validation and
  persistence are in the backend. This keeps the UI thin and the core testable.
- **Repository traits.** Services depend on `GameRepository`, not on SQLite directly,
  enabling future storage changes and unit testing with fakes.
- **Database access is abstracted.** The application must access SQLite only through a dedicated database abstraction (DatabaseManager or equivalent). The current implementation may internally use a single Mutex<Connection>, but the rest of the application must not depend on that implementation detail. This allows replacing the storage strategy in the future without affecting services or repositories.
- **Genres are normalized, not comma-strings.** Reusable `genres` entities with a
  `game_genres` join table. `GenreRepository` owns writes to both genre tables
  (`get_or_create` dedupes by normalized name; `replace_for_game` re-links a game's set);
  `GameRepository` reads them when hydrating/filtering the game aggregate. This keeps the
  door open for genre stats, filtering and management with no schema change.
- **One flexible query, not many endpoints.** The game browser is driven by a single
  `GameQuery`; `build_filters` turns it into parameterized SQL. Adding a filter is a local
  change there. The aggregate `Game` (with `genres`) is assembled in `GameService` writes
  inside a transaction so game row + genre links commit atomically.
- **`bundled` SQLite.** No system SQLite dependency; reproducible builds.
- **Hand-rolled migration runner.** `user_version`-based, zero extra dependencies, easy to
  reason about. Append-only — never edit an applied migration.
- **Icons** are the white-and-blue stylized X. The macOS-facing assets carry genuine
  **transparent squircle corners** (an alpha mask with ~7% margin and ~22% corner radius —
  macOS does not round Dock icons itself, so the PNG must already be shaped):
  `icon-source.png` (base) and `icon-playing.png` (runtime green Play-badged state) are the
  Dock icons embedded via `include_bytes!`; `icon.icns` is the bundle icon, built from the
  rounded base with `sips` + `iconutil` (a `.iconset` of 16→1024px). The base Dock icon is
  applied at launch in `lib.rs` `setup` (`set_playing_icon(false)`) so it always shows in
  dev, not only after a session starts. Runtime decoding uses Tauri's `image-png` feature.
  - **Do not** blindly rerun `npm run tauri -- icon …` from the rounded source: it would
    propagate the transparent corners into the iOS/Android/Windows icons, which apply their
    own platform masking and expect a full-bleed square. The desktop PNGs (`32/64/128/
    128@2x`, `icon.png`) and `icon.icns` are the macOS/Linux set kept in sync with the seed;
    the `Square*Logo.png`, `ios/`, and `android/` assets stay square. Copy the `128x128.png`
    to `public/xareon-icon.png` so the sidebar brand matches.

## 9a. Cross-platform requirements

Xareon targets both **macOS and Windows**. macOS is the current primary development
environment, but Windows is a required supported platform rather than a future optional
port. Before implementing any feature that touches the operating system, an agent must
explicitly assess whether its APIs, behavior, assets, permissions, paths, lifecycle, or
UX differ between macOS and Windows.

The cross-platform rules are mandatory:

- Keep domain logic, services, repositories, database behavior, and frontend contracts
  platform-independent. Isolate only the smallest necessary system-integration code.
- Use Rust compile-time branches such as `#[cfg(target_os = "macos")]` and
  `#[cfg(target_os = "windows")]` when native implementations differ. Do not scatter
  platform checks through shared business logic.
- Put OS-specific crates in target-specific Cargo dependency sections. A macOS-only or
  Windows-only crate must never become an unconditional dependency.
- Implement behavior for both target operating systems when the feature is expected to
  work on both. A no-op, ignored call, or generic fallback is not automatically a valid
  Windows implementation; confirm that it provides the intended user-visible behavior.
- Check Tauri and native API semantics per platform. Similar names do not imply similar
  effects—for example, a window icon, macOS Dock icon, and Windows taskbar icon are
  distinct system concepts.
- Account for platform differences in filesystem paths, app-data directories, path
  separators, process and window lifecycle, Dock/taskbar integration, icons and bundle
  formats, keyboard shortcuts, permissions, notifications, and native dialogs.
- Prefer Tauri's verified cross-platform API when it provides equivalent behavior. Use
  native APIs behind a small platform adapter when Tauri does not expose the required
  semantics on one or both platforms.
- Validate the current host build and, whenever the toolchain is available, compile-check
  the other target too. Never claim Windows behavior was tested when only macOS was
  tested. Record unverified platform behavior clearly in the final handoff and in this
  file when it is an ongoing limitation.
- Do not break one platform while fixing the other. Changes to OS integration must
  preserve a compilable branch for every supported target and a sensible unsupported-OS
  fallback where applicable.
- Update this section or the relevant module documentation whenever a new platform-
  specific integration or limitation is introduced.

## 10. Known limitations

- Implemented: **Games** (CRUD + browser query), **Genres** (multi, normalized), manual
  and Windows-only automatic
  **Play tracking** (single active session, history and cached total), the
  per-game **Journal**, per-game **Achievements**, **Statistics**, **Settings**, and manual
  profile backup/restore through a Google Drive-synchronized local folder. Not yet built:
  personal tags, screenshot gallery, timeline, global achievements dashboard.
- Dates are stored as plain ISO/`datetime('now')` strings (`TEXT`, UTC); no calendar or
  timezone handling beyond formatting in the UI.
- The browser query has no pagination yet (fine for a personal library; revisit if needed).
- Automated coverage currently includes focused profile backup/status/restore tests; most
  services still rely on generic repository boundaries for future fake-based tests.
- Sync is intentionally manual and whole-database: it cannot merge independently edited
  Mac and Windows databases. Wait for Google Drive desktop sync between Upload and Restore;
  a detected divergence requires consciously choosing which whole copy wins.
- The global-shortcut backend has built and launched on Windows. A restored Mac shortcut
  previously collided with Windows' reserved `Win+Shift+S`; shortcuts are now per-device
  and startup registration failures are non-fatal. This corrective revision is compile-
  checked on macOS and still needs one confirmation run on Windows hardware.
- The Windows system tray cannot show a persistent text title beside its icon; current
  session duration is therefore available in its tooltip. This behavior is implemented
  through Tauri's cross-platform tray API but is not yet exercised on Windows hardware.
- Automatic tracking is compile-checked on Windows, but foreground/exclusive-fullscreen,
  protected-process, sleep/resume and AFK behavior still needs an end-to-end run with real
  games on Windows hardware.

## 11. How to add a new feature

Adding a module (e.g. Journal) end-to-end:

0. **Platform assessment:** identify any macOS/Windows differences and decide whether a
   shared Tauri API or isolated target-specific adapters are required.
1. **Migration:** add `src-tauri/src/migrations/000X_*.sql` and register it (in order) in
   `db/migrations.rs`. Use foreign keys to `games(id)` where relevant.
2. **Domain:** add models in `src-tauri/src/domain/` (camelCase serde for wire types).
3. **Repository:** define a `…Repository` trait + SQLite impl in
   `src-tauri/src/repositories/`. **All SQL goes here.**
4. **Service:** add a `…Service` in `src-tauri/src/services/` for validation/rules.
5. **Commands:** add `#[tauri::command]` handlers in `src-tauri/src/commands/` and register
   them in `lib.rs` `generate_handler!`.
5a. If the feature stores files (covers, screenshots, exports, backups), implement it through the storage layer instead of using filesystem APIs directly from services or commands.
6. **Frontend types:** mirror the domain in `src/types/`.
7. **Frontend api:** add a typed wrapper in `src/api/`.
8. **Frontend view:** add a view in `src/views/` and enable its nav entry in `main.ts`.
9. **Update this file** (§3, §6, §7, §10) to match.

## 12. Roadmap

Done: journal entries with date/time; multi-genre (normalized); search, filtering &
sorting; centralized settings; per-game user-defined achievements with optional progress;
statistics; manual whole-profile backup/restore via a local cloud-synced folder.

Specified initial scope still to build:
- Timeline view (cross-game).
- Personal tags.
- Screenshot gallery.
- Global achievements dashboard (per-game custom achievements already exist).

Future expansion (from the spec):
- Steam library import.
- Automatic or merge-capable cloud sync (manual whole-profile transfer exists).
- Plugins.
- Data export/import.
- Localization (multi-language support).
- Theme manager (support additional themes in the future).
- Automatic backups.
- Logging and diagnostics.

## 13. Development principles

- Architecture stability is more important than adding new features.
- Treat macOS and Windows as first-class targets; every OS-facing change requires an
  explicit cross-platform assessment and platform-appropriate implementation.
- Every new feature must fit into the existing architecture.
- Never bypass architectural layers.
- Never place business logic in the UI or command layer.
- Prefer extending existing modules over creating new abstractions.
- Before adding a new dependency, first evaluate whether the same result can be achieved using the Rust standard library or existing project dependencies.
- Keep dependencies to a minimum.
- Journal entries (Diary Entries) are one of the core concepts of Xareon and should always be treated as a first-class feature rather than an auxiliary notes system.
- Whenever possible, keep the project understandable for future AI agents as well as human developers by maintaining clear module boundaries and keeping AGENTS.md up to date.
