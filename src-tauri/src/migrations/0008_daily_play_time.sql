BEGIN;

CREATE TABLE active_play_session (
    singleton_id     INTEGER PRIMARY KEY CHECK (singleton_id = 1),
    game_id          INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    started_at       TEXT NOT NULL,
    last_activity_at TEXT NOT NULL,
    tracking_source  TEXT NOT NULL CHECK (tracking_source IN ('manual', 'automatic'))
);

CREATE TABLE daily_play_time (
    game_id           INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    play_date         TEXT NOT NULL,
    duration_seconds  INTEGER NOT NULL CHECK (duration_seconds >= 0),
    manual_seconds    INTEGER NOT NULL CHECK (manual_seconds >= 0),
    automatic_seconds INTEGER NOT NULL CHECK (automatic_seconds >= 0),
    sessions_count    INTEGER NOT NULL CHECK (sessions_count > 0),
    first_started_at  TEXT NOT NULL,
    last_ended_at     TEXT NOT NULL,
    updated_at        TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (game_id, play_date),
    CHECK (duration_seconds = manual_seconds + automatic_seconds)
);

CREATE INDEX daily_play_time_date ON daily_play_time (play_date);

-- Preserve an interrupted session as the new singleton runtime state. Startup
-- recovery will close it at its last heartbeat after migrations finish.
INSERT INTO active_play_session (
    singleton_id, game_id, started_at, last_activity_at, tracking_source
)
SELECT 1, game_id, started_at, last_activity_at, tracking_source
FROM play_sessions
WHERE ended_at IS NULL;

-- Split every completed legacy interval at local-midnight boundaries. SQLite's
-- `utc` modifier converts each local boundary back to UTC, so DST transitions
-- and sessions crossing midnight retain their exact elapsed seconds.
WITH RECURSIVE legacy_days (
    session_id, game_id, tracking_source, started_epoch, ended_epoch, play_date
) AS (
    SELECT
        id,
        game_id,
        tracking_source,
        CAST(strftime('%s', started_at) AS INTEGER),
        CAST(strftime('%s', ended_at) AS INTEGER),
        date(started_at, 'localtime')
    FROM play_sessions
    WHERE ended_at IS NOT NULL

    UNION ALL

    SELECT
        session_id,
        game_id,
        tracking_source,
        started_epoch,
        ended_epoch,
        date(play_date, '+1 day')
    FROM legacy_days
    WHERE play_date < date(ended_epoch, 'unixepoch', 'localtime')
), legacy_segments AS (
    SELECT
        session_id,
        game_id,
        tracking_source,
        play_date,
        MAX(started_epoch, CAST(strftime('%s', play_date, 'utc') AS INTEGER)) AS segment_start,
        MIN(ended_epoch, CAST(strftime('%s', play_date, '+1 day', 'utc') AS INTEGER)) AS segment_end
    FROM legacy_days
), positive_segments AS (
    SELECT * FROM legacy_segments WHERE segment_end > segment_start
)
INSERT INTO daily_play_time (
    game_id, play_date, duration_seconds, manual_seconds, automatic_seconds,
    sessions_count, first_started_at, last_ended_at
)
SELECT
    game_id,
    play_date,
    SUM(segment_end - segment_start),
    SUM(CASE WHEN tracking_source = 'manual' THEN segment_end - segment_start ELSE 0 END),
    SUM(CASE WHEN tracking_source = 'automatic' THEN segment_end - segment_start ELSE 0 END),
    COUNT(DISTINCT session_id),
    datetime(MIN(segment_start), 'unixepoch'),
    datetime(MAX(segment_end), 'unixepoch')
FROM positive_segments
GROUP BY game_id, play_date;

DROP TABLE play_sessions;

COMMIT;
