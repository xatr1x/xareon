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
│   │   └── games.ts            # typed wrappers over game_* commands
│   ├── types/
│   │   └── game.ts             # mirror of Rust Game/GameInput/GameStatus
│   ├── ui/
│   │   └── dom.ts              # tiny typed DOM helpers (el, clear)
│   └── views/
│       ├── games-view.ts       # games list + table
│       └── game-form.ts        # create/edit modal form
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
        │   └── game.rs         # Game, GameInput, GameStatus
        ├── repositories/
        │   └── game_repository.rs  # GameRepository trait + SQLite impl (SQL lives here)
        ├── services/
        │   └── game_service.rs # GameService (validation + orchestration)
        ├── validation/         # reusable business validation rules
        │   └── mod.rs          # require_non_empty, require_in_range
        ├── storage/            # file storage (covers, screenshots, backups) — reserved
        ├── config/             # application configuration — reserved
        ├── events/             # domain events — reserved for future use
        ├── commands/
        │   └── game_commands.rs    # #[tauri::command] handlers
        ├── db/
        │   ├── connection.rs   # open() + enable FKs + run migrations
        │   ├── manager.rs      # DatabaseManager — sole gateway to the connection
        │   └── migrations.rs   # versioned migration runner (user_version)
        └── migrations/
            └── 0001_init.sql   # games table
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

### Schema (current)

`games`
| column        | type    | notes                                                                 |
|---------------|---------|-----------------------------------------------------------------------|
| id            | INTEGER | PK, autoincrement                                                     |
| title         | TEXT    | NOT NULL                                                              |
| genre         | TEXT    | nullable                                                              |
| platform      | TEXT    | nullable                                                              |
| developer     | TEXT    | nullable                                                              |
| publisher     | TEXT    | nullable                                                              |
| release_year  | INTEGER | nullable                                                              |
| started_at    | TEXT    | nullable, ISO date                                                   |
| finished_at   | TEXT    | nullable, ISO date                                                   |
| status        | TEXT    | NOT NULL, default `planned`, CHECK in status set                     |
| rating        | INTEGER | nullable, CHECK 0–10                                                  |
| cover_path    | TEXT    | nullable                                                              |
| created_at    | TEXT    | NOT NULL, default `datetime('now')`                                  |
| updated_at    | TEXT    | NOT NULL, default `datetime('now')`; set on update                  |

**Game statuses:** `planned`, `playing`, `paused`, `completed`, `completed_100`, `dropped`.

## 7. Modules (current)

- **Games** — full CRUD. Commands: `list_games`, `get_game`, `create_game`,
  `update_game`, `delete_game`.

The frontend navigation already lists future modules (Timeline, Achievements, Statistics)
as disabled placeholders.

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
- **Comments:** only where they add value. Consistent naming. No duplicated code.

## 9. Important architectural decisions

- **Logic in Rust, not TypeScript.** The frontend is a pure UI; all rules, validation and
  persistence are in the backend. This keeps the UI thin and the core testable.
- **Repository traits.** Services depend on `GameRepository`, not on SQLite directly,
  enabling future storage changes and unit testing with fakes.
- **Database access is abstracted.** The application must access SQLite only through a dedicated database abstraction (DatabaseManager or equivalent). The current implementation may internally use a single Mutex<Connection>, but the rest of the application must not depend on that implementation detail. This allows replacing the storage strategy in the future without affecting services or repositories.
- **`bundled` SQLite.** No system SQLite dependency; reproducible builds.
- **Hand-rolled migration runner.** `user_version`-based, zero extra dependencies, easy to
  reason about. Append-only — never edit an applied migration.
- **Icons** are generated from a seed image via `npm run tauri icon` and committed under
  `src-tauri/icons/`. Replace the seed and regenerate to rebrand.

## 10. Known limitations

- Only the **Games** module is implemented (MVP). Journal, tags, screenshots, achievements
  and statistics are specified but not yet built.
- Dates are stored as plain ISO strings (`TEXT`); no calendar/timezone handling.
- No automated tests yet (the service layer is structured to allow them).
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

Specified initial scope still to build:
- Personal journal entries with date/time.
- Timeline view.
- Personal tags.
- Screenshot gallery.
- Universal achievements system (per game; custom achievements; progress; hidden flag).
- Statistics page.

Future expansion (from the spec):
- Steam library import.
- Backup & restore.
- Cloud sync.
- Plugins.
- Playtime tracking.
- Search & filtering.
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
