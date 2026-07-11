ALTER TABLE games ADD COLUMN total_play_time_seconds INTEGER NOT NULL DEFAULT 0 CHECK (total_play_time_seconds >= 0);
ALTER TABLE games ADD COLUMN is_playing_now INTEGER NOT NULL DEFAULT 0 CHECK (is_playing_now IN (0, 1));
ALTER TABLE games ADD COLUMN last_played_at TEXT;

CREATE TABLE play_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    last_activity_at TEXT NOT NULL,
    duration_seconds INTEGER CHECK (duration_seconds IS NULL OR duration_seconds >= 0),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK ((ended_at IS NULL AND duration_seconds IS NULL) OR
           (ended_at IS NOT NULL AND duration_seconds IS NOT NULL))
);

-- A constant expression makes this a database-wide singleton, not merely one per game.
CREATE UNIQUE INDEX one_active_play_session ON play_sessions ((1)) WHERE ended_at IS NULL;
CREATE INDEX play_sessions_game_started ON play_sessions (game_id, started_at DESC);
CREATE INDEX play_sessions_started ON play_sessions (started_at);

CREATE TRIGGER prevent_deleting_actively_played_game
BEFORE DELETE ON games
WHEN OLD.is_playing_now = 1
BEGIN
    SELECT RAISE(ABORT, 'stop the active play session before deleting this game');
END;
