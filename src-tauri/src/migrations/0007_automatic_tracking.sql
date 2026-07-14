ALTER TABLE games ADD COLUMN automatic_tracking_enabled INTEGER NOT NULL DEFAULT 0 CHECK (automatic_tracking_enabled IN (0, 1));

CREATE TABLE game_executable_bindings (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id               INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    executable_path       TEXT NOT NULL,
    executable_normalized TEXT NOT NULL UNIQUE,
    executable_name       TEXT NOT NULL,
    created_at            TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX game_executable_bindings_game ON game_executable_bindings (game_id);

ALTER TABLE play_sessions ADD COLUMN tracking_source TEXT NOT NULL DEFAULT 'manual'
    CHECK (tracking_source IN ('manual', 'automatic'));
ALTER TABLE play_sessions ADD COLUMN ended_reason TEXT
    CHECK (ended_reason IS NULL OR ended_reason IN ('manual', 'process_exit', 'afk', 'shutdown', 'recovery'));
