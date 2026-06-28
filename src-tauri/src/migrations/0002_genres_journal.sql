-- Normalize genres into reusable entities with a many-to-many link to games,
-- and add the per-game journal.

-- Reusable genre entities. `name_normalized` (lowercased, trimmed) is the unique
-- key used for recognition/deduplication; `name` keeps the first-seen casing.
CREATE TABLE genres (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    name_normalized TEXT NOT NULL UNIQUE
);

-- Many-to-many: a game has many genres, a genre belongs to many games.
CREATE TABLE game_genres (
    game_id  INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    genre_id INTEGER NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    PRIMARY KEY (game_id, genre_id)
);

CREATE INDEX idx_game_genres_genre ON game_genres (genre_id);

-- Migrate existing single-value `games.genre` into the normalized tables, then
-- drop the old column.
INSERT INTO genres (name, name_normalized)
SELECT DISTINCT trim(genre), lower(trim(genre))
FROM games
WHERE genre IS NOT NULL AND trim(genre) <> ''
ON CONFLICT(name_normalized) DO NOTHING;

INSERT INTO game_genres (game_id, genre_id)
SELECT g.id, ge.id
FROM games g
JOIN genres ge ON ge.name_normalized = lower(trim(g.genre))
WHERE g.genre IS NOT NULL AND trim(g.genre) <> '';

ALTER TABLE games DROP COLUMN genre;

-- Per-game journal. A first-class entity, not a comment. Extra metadata
-- (mood, tags, screenshots) can be added later via new columns/tables.
CREATE TABLE journal_entries (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id    INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    body       TEXT    NOT NULL,
    created_at TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- Newest-first listing per game.
CREATE INDEX idx_journal_game_created ON journal_entries (game_id, created_at DESC, id DESC);
