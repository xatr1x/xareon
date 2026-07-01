CREATE TABLE achievements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    category TEXT,
    status TEXT NOT NULL DEFAULT 'planned'
        CHECK (status IN ('planned', 'in_progress', 'completed')),
    progress_current INTEGER CHECK (progress_current IS NULL OR progress_current >= 0),
    progress_target INTEGER CHECK (progress_target IS NULL OR progress_target > 0),
    progress_unit TEXT,
    completed_at TEXT,
    is_hidden INTEGER NOT NULL DEFAULT 0 CHECK (is_hidden IN (0, 1)),
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    CHECK (
        progress_current IS NULL
        OR progress_target IS NULL
        OR progress_current <= progress_target
    )
);

CREATE INDEX idx_achievements_game_id ON achievements(game_id);
CREATE INDEX idx_achievements_game_category ON achievements(game_id, category);
