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
- **storage** — file storage for covers/screenshots/backups (reserved; see §11 step 5a).
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
├── tsconfig.json               # strict TypeScript config
├── vite.config.ts              # Vite/Tauri dev server config
├── AGENTS.md                   # this file
├── idea.md                     # original product specification
├── src/                        # frontend (TypeScript, UI only)
│   ├── main.ts                 # app shell + sidebar navigation
│   ├── styles.css              # dark, minimal theme
│   ├── api/
│   │   ├── achievements.ts     # wrappers over achievement commands
│   │   ├── games.ts            # typed wrappers over game_* + list_genres
│   │   ├── journal.ts          # wrappers over *_journal_entry commands
│   │   └── settings.ts         # wrappers over get_settings/update_settings
│   ├── types/
│   │   ├── achievement.ts      # Achievement/AchievementStatus + input types
│   │   ├── game.ts             # Game/GameInput/GameStatus + GameQuery/sort types
│   │   ├── genre.ts            # Genre
│   │   ├── journal.ts          # JournalEntry + inputs
│   │   └── settings.ts         # Settings
│   ├── ui/
│   │   ├── dom.ts              # tiny typed DOM helpers (el, clear)
│   │   └── format.ts           # date/time formatting
│   └── views/
│       ├── games-view.ts       # game browser: filters, sort, table
│       ├── game-form.ts        # create/edit modal form (multi-genre input)
│       ├── game-detail.ts      # tabbed game detail (overview/achievements/journal/details)
│       └── settings-view.ts    # settings page (load + save)
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
        │   ├── game.rs         # Game, GameInput, GameStatus, GameQuery + sort/filter enums
        │   ├── genre.rs        # Genre + normalize()
        │   ├── journal.rs      # JournalEntry, NewJournalEntry, JournalEntryUpdate
        │   └── settings.rs     # Settings (typed aggregate of app settings)
        ├── repositories/
        │   ├── achievement_repository.rs # achievements table
        │   ├── game_repository.rs     # games table + browser query + genre hydration
        │   ├── genre_repository.rs    # genres + game_genres writes (get_or_create, links)
        │   ├── journal_repository.rs  # journal_entries
        │   └── settings_repository.rs # settings key-value store (get_all/set)
        ├── services/
        │   ├── achievement_service.rs # validation + progress/status rules
        │   ├── game_service.rs     # GameService (validation + game/genre orchestration)
        │   ├── genre_service.rs    # GenreService (list genres)
        │   ├── journal_service.rs  # JournalService (validation, ensures game exists)
        │   └── settings_service.rs # SettingsService (maps typed Settings ↔ KV keys)
        ├── validation/         # reusable business validation rules
        │   └── mod.rs          # require_non_empty, require_in_range
        ├── storage/            # file storage (covers, screenshots, backups) — reserved
        ├── config/            # application configuration — reserved
        ├── events/            # domain events — reserved for future use
        ├── commands/
        │   ├── achievement_commands.rs # achievement #[tauri::command] handlers
        │   ├── game_commands.rs     # game #[tauri::command] handlers (writes in a tx)
        │   ├── genre_commands.rs    # list_genres
        │   ├── journal_commands.rs  # journal #[tauri::command] handlers
        │   └── settings_commands.rs # get_settings, update_settings (write in a tx)
        ├── db/
        │   ├── connection.rs   # open() + enable FKs + run migrations
        │   ├── manager.rs      # DatabaseManager — sole gateway to the connection
        │   └── migrations.rs   # versioned migration runner (user_version)
        └── migrations/
            ├── 0001_init.sql            # games table
            ├── 0002_genres_journal.sql  # genres, game_genres, journal_entries
            ├── 0003_settings.sql        # settings key-value store
            └── 0004_achievements.sql    # user-defined personal achievements
```

## 4. Technology stack

- **Tauri v2** — desktop shell.
- **Rust** — backend / all application logic.
- **rusqlite** (`bundled` feature) — SQLite driver; SQLite is compiled in, no system dep.
- **serde / serde_json** — (de)serialization across the command boundary.
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

**Game statuses:** `planned`, `playing`, `paused`, `completed`, `completed_100`, `dropped`.
**Achievement statuses:** `planned`, `in_progress`, `completed`.

## 7. Modules (current)

- **Games** — full CRUD plus a flexible browser query. Commands: `list_games` (takes an
  optional `GameQuery`), `get_game`, `create_game`, `update_game`, `delete_game`.
  - The browser table shows a compact **Time** column instead of platform: completed
    date ranges render as a calendar duration (for example, `1 mo, 3 d`), while playing
    games show time since their start date (for example, `Yesterday`). Platform remains
    available in the game form, detail view, and advanced browser filter.
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

- **Settings** — a centralized, extensible settings system stored in SQLite as a
  key-value store, designed to grow as future features need configuration.
  Commands: `get_settings`, `update_settings` (loads/saves the whole `Settings`
  aggregate). The typed `Settings` model maps to KV keys in `SettingsService` (the
  single mapping point); adding a setting is a field on `Settings` + a key there —
  **no migration**. First settings: `userIdentifier` (human-readable public
  handle, also the future Google Drive folder name; not a UUID) and
  `googleDriveFolder` (URL stored now for the future sync system — the Drive
  integration itself is not implemented). Saving runs inside a transaction so all
  settings commit atomically.

The frontend navigation lists future global modules (Timeline, Achievements,
Statistics) as disabled placeholders. Per-game achievements are live inside game details;
there is not yet a global achievements dashboard. Settings is a live nav entry.

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
- **Icons** are generated from a seed image via `npm run tauri icon` and committed under
  `src-tauri/icons/`. Replace the seed and regenerate to rebrand.

## 10. Known limitations

- Implemented: **Games** (CRUD + browser query), **Genres** (multi, normalized), the
  per-game **Journal**, per-game **Achievements**, and **Settings** (user identifier +
  Google Drive folder URL). Not yet built: personal tags, screenshot gallery,
  statistics, timeline, global achievements dashboard.
- Dates are stored as plain ISO/`datetime('now')` strings (`TEXT`, UTC); no calendar or
  timezone handling beyond formatting in the UI.
- The browser query has no pagination yet (fine for a personal library; revisit if needed).
- No automated tests yet (services are generic over repository traits to allow fakes).
- Single local database; no backup/restore, sync, or import.

## 11. How to add a new feature

Adding a module (e.g. Journal) end-to-end:

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
sorting; centralized settings (user identifier + Google Drive folder URL); per-game
user-defined achievements with optional progress.

Specified initial scope still to build:
- Timeline view (cross-game).
- Personal tags.
- Screenshot gallery.
- Global achievements dashboard (per-game custom achievements already exist).
- Statistics page (incl. genre statistics, enabled by the normalized genres).

Future expansion (from the spec):
- Steam library import.
- Backup & restore.
- Cloud sync.
- Plugins.
- Playtime tracking.
- Data export/import.
- Localization (multi-language support).
- Theme manager (support additional themes in the future).
- Automatic backups.
- Logging and diagnostics.

## 13. Development principles

- Architecture stability is more important than adding new features.
- Every new feature must fit into the existing architecture.
- Never bypass architectural layers.
- Never place business logic in the UI or command layer.
- Prefer extending existing modules over creating new abstractions.
- Before adding a new dependency, first evaluate whether the same result can be achieved using the Rust standard library or existing project dependencies.
- Keep dependencies to a minimum.
- Journal entries (Diary Entries) are one of the core concepts of Xareon and should always be treated as a first-class feature rather than an auxiliary notes system.
- Whenever possible, keep the project understandable for future AI agents as well as human developers by maintaining clear module boundaries and keeping AGENTS.md up to date.
