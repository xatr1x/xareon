-- Initial schema: the games table.
CREATE TABLE games (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    title        TEXT    NOT NULL,
    genre        TEXT,
    platform     TEXT,
    developer    TEXT,
    publisher    TEXT,
    release_year INTEGER,
    started_at   TEXT,
    finished_at  TEXT,
    status       TEXT    NOT NULL DEFAULT 'planned'
                 CHECK (status IN ('planned', 'playing', 'paused', 'completed', 'completed_100', 'dropped')),
    rating       INTEGER CHECK (rating IS NULL OR (rating BETWEEN 0 AND 10)),
    cover_path   TEXT,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_games_status ON games (status);
