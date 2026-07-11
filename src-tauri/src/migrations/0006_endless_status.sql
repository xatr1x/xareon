-- Add the 'endless' game status for evergreen games that have no ending
-- (MMOs, roguelikes, live-service, sandboxes). SQLite cannot ALTER a CHECK
-- constraint, so rebuild the games table with the widened CHECK, preserving all
-- rows and re-creating the index and the delete-guard trigger.
--
-- foreign_keys must be toggled OUTSIDE a transaction to take effect, so the OFF
-- pragma precedes BEGIN. With enforcement off, DROP TABLE does not disturb the
-- child rows (game_genres, journal_entries, achievements, play_sessions), which
-- reference games by name and keep their preserved ids.
PRAGMA foreign_keys=OFF;

BEGIN;

CREATE TABLE games_new (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    title        TEXT    NOT NULL,
    platform     TEXT,
    developer    TEXT,
    publisher    TEXT,
    release_year INTEGER,
    started_at   TEXT,
    finished_at  TEXT,
    status       TEXT    NOT NULL DEFAULT 'planned'
                 CHECK (status IN ('planned', 'playing', 'paused', 'completed', 'completed_100', 'dropped', 'endless')),
    rating       INTEGER CHECK (rating IS NULL OR (rating BETWEEN 0 AND 10)),
    cover_path   TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    total_play_time_seconds INTEGER NOT NULL DEFAULT 0 CHECK (total_play_time_seconds >= 0),
    is_playing_now INTEGER NOT NULL DEFAULT 0 CHECK (is_playing_now IN (0, 1)),
    last_played_at TEXT
);

INSERT INTO games_new (
    id, title, platform, developer, publisher, release_year, started_at, finished_at,
    status, rating, cover_path, created_at, updated_at,
    total_play_time_seconds, is_playing_now, last_played_at
)
SELECT
    id, title, platform, developer, publisher, release_year, started_at, finished_at,
    status, rating, cover_path, created_at, updated_at,
    total_play_time_seconds, is_playing_now, last_played_at
FROM games;

DROP TABLE games;
ALTER TABLE games_new RENAME TO games;

CREATE INDEX idx_games_status ON games (status);

-- Re-create the guard from 0005 (also repairs DBs where it drifted away).
CREATE TRIGGER prevent_deleting_actively_played_game
BEFORE DELETE ON games
WHEN OLD.is_playing_now = 1
BEGIN
    SELECT RAISE(ABORT, 'stop the active play session before deleting this game');
END;

COMMIT;

PRAGMA foreign_keys=ON;
