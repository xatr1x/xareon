use std::collections::HashMap;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};

use crate::domain::game::{
    Game, GameInput, GameQuery, GameSort, GameStatus, GenreMatch, SortDirection,
};
use crate::domain::genre::normalize;
use crate::error::{AppError, AppResult};

/// Persistence operations for the `games` table and the game browser query.
/// Reads hydrate each game's `genres` (from `game_genres`/`genres`); writes here
/// cover only the `games` row — genre links are owned by `GenreRepository`.
pub trait GameRepository {
    fn query(&self, query: &GameQuery) -> AppResult<Vec<Game>>;
    fn get(&self, id: i64) -> AppResult<Game>;
    fn create(&self, input: &GameInput) -> AppResult<i64>;
    fn update(&self, id: i64, input: &GameInput) -> AppResult<()>;
    fn delete(&self, id: i64) -> AppResult<()>;
}

/// Columns selected when reading a full game row, in `map_row` order. `genres`
/// is not a column — it is hydrated separately.
const COLUMNS: &str = "id, title, platform, developer, publisher, release_year, \
    started_at, finished_at, status, rating, cover_path, total_play_time_seconds, \
    is_playing_now, last_played_at, \
    (SELECT started_at FROM play_sessions ps WHERE ps.game_id = games.id AND ps.ended_at IS NULL) \
    AS active_session_started_at, created_at, updated_at";

pub struct SqliteGameRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteGameRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn map_row(row: &Row<'_>) -> rusqlite::Result<Game> {
        Ok(Game {
            id: row.get("id")?,
            title: row.get("title")?,
            genres: Vec::new(), // hydrated by `hydrate_genres`
            platform: row.get("platform")?,
            developer: row.get("developer")?,
            publisher: row.get("publisher")?,
            release_year: row.get("release_year")?,
            started_at: row.get("started_at")?,
            finished_at: row.get("finished_at")?,
            status: row.get("status")?,
            rating: row.get("rating")?,
            cover_path: row.get("cover_path")?,
            total_play_time_seconds: row.get("total_play_time_seconds")?,
            is_playing_now: row.get("is_playing_now")?,
            last_played_at: row.get("last_played_at")?,
            active_session_started_at: row.get("active_session_started_at")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }

    /// Populate `genres` for the given games with a single grouped query.
    fn hydrate_genres(&self, games: &mut [Game]) -> AppResult<()> {
        if games.is_empty() {
            return Ok(());
        }
        let ids: Vec<Value> = games.iter().map(|g| Value::Integer(g.id)).collect();
        let placeholders = vec!["?"; ids.len()].join(", ");
        let sql = format!(
            "SELECT gg.game_id, ge.name FROM game_genres gg \
             JOIN genres ge ON ge.id = gg.genre_id \
             WHERE gg.game_id IN ({placeholders}) \
             ORDER BY ge.name COLLATE NOCASE"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut by_game: HashMap<i64, Vec<String>> = HashMap::new();
        let rows = stmt.query_map(params_from_iter(ids), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (game_id, name) = row?;
            by_game.entry(game_id).or_default().push(name);
        }
        for game in games.iter_mut() {
            if let Some(names) = by_game.remove(&game.id) {
                game.genres = names;
            }
        }
        Ok(())
    }
}

impl GameRepository for SqliteGameRepository<'_> {
    fn query(&self, query: &GameQuery) -> AppResult<Vec<Game>> {
        let (where_sql, params) = build_filters(query);
        let sql = format!(
            "SELECT {COLUMNS} FROM games{where_sql} ORDER BY {}",
            order_clause(query.sort, query.direction)
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut games = stmt
            .query_map(params_from_iter(params), Self::map_row)?
            .collect::<rusqlite::Result<Vec<Game>>>()?;
        self.hydrate_genres(&mut games)?;
        Ok(games)
    }

    fn get(&self, id: i64) -> AppResult<Game> {
        let game = self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM games WHERE id = ?1"),
                [id],
                Self::map_row,
            )
            .optional()?;
        let mut game = game.ok_or(AppError::NotFound)?;
        self.hydrate_genres(std::slice::from_mut(&mut game))?;
        Ok(game)
    }

    fn create(&self, input: &GameInput) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO games \
             (title, platform, developer, publisher, release_year, \
              started_at, finished_at, status, rating, cover_path) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                input.title,
                input.platform,
                input.developer,
                input.publisher,
                input.release_year,
                input.started_at,
                input.finished_at,
                input.status,
                input.rating,
                input.cover_path,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    fn update(&self, id: i64, input: &GameInput) -> AppResult<()> {
        let affected = self.conn.execute(
            "UPDATE games SET \
             title = ?1, platform = ?2, developer = ?3, publisher = ?4, \
             release_year = ?5, started_at = ?6, finished_at = ?7, status = ?8, \
             rating = ?9, cover_path = ?10, updated_at = datetime('now') \
             WHERE id = ?11",
            params![
                input.title,
                input.platform,
                input.developer,
                input.publisher,
                input.release_year,
                input.started_at,
                input.finished_at,
                input.status,
                input.rating,
                input.cover_path,
                id,
            ],
        )?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    fn delete(&self, id: i64) -> AppResult<()> {
        let affected = self.conn.execute("DELETE FROM games WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}

/// Build the `WHERE` clause and its bound parameters from a query. Returns an
/// empty string when there are no filters. Adding a filter means adding a branch
/// here — the rest of the stack is untouched.
fn build_filters(query: &GameQuery) -> (String, Vec<Value>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<Value> = Vec::new();

    if let Some(search) = query.search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        clauses.push("title LIKE ?".to_string());
        params.push(Value::Text(format!("%{search}%")));
    }

    if !query.statuses.is_empty() {
        let placeholders = push_text_list(&mut params, query.statuses.iter().map(|s| s.as_str()));
        clauses.push(format!("status IN ({placeholders})"));
    }

    if !query.platforms.is_empty() {
        let placeholders = push_text_list(
            &mut params,
            query.platforms.iter().map(String::as_str).map(str::trim).filter(|s| !s.is_empty()),
        );
        if !placeholders.is_empty() {
            clauses.push(format!("platform IN ({placeholders})"));
        }
    }

    let genres: Vec<String> = query
        .genres
        .iter()
        .map(|g| normalize(g))
        .filter(|g| !g.is_empty())
        .collect();
    if !genres.is_empty() {
        let placeholders = push_text_list(&mut params, genres.iter().map(String::as_str));
        let exists = format!(
            "SELECT 1 FROM game_genres gg JOIN genres ge ON ge.id = gg.genre_id \
             WHERE gg.game_id = games.id AND ge.name_normalized IN ({placeholders})"
        );
        match query.genre_match {
            GenreMatch::Any => clauses.push(format!("EXISTS ({exists})")),
            GenreMatch::All => {
                clauses.push(format!(
                    "(SELECT COUNT(DISTINCT ge.name_normalized) FROM game_genres gg \
                     JOIN genres ge ON ge.id = gg.genre_id \
                     WHERE gg.game_id = games.id AND ge.name_normalized IN ({placeholders})) = ?"
                ));
                params.push(Value::Integer(genres.len() as i64));
            }
        }
    }

    if let Some(year) = query.release_year {
        clauses.push("release_year = ?".to_string());
        params.push(Value::Integer(year));
    }
    if let Some(year) = query.started_year {
        clauses.push("substr(started_at, 1, 4) = ?".to_string());
        params.push(Value::Text(format!("{year:04}")));
    }
    if let Some(year) = query.finished_year {
        clauses.push("substr(finished_at, 1, 4) = ?".to_string());
        params.push(Value::Text(format!("{year:04}")));
    }
    if let Some(year) = query.played_year {
        clauses.push(
            "(started_at IS NOT NULL AND substr(started_at, 1, 4) <= ? \
             AND (finished_at IS NULL OR substr(finished_at, 1, 4) >= ?))"
                .to_string(),
        );
        params.push(Value::Text(format!("{year:04}")));
        params.push(Value::Text(format!("{year:04}")));
    }
    if let Some(min) = query.min_rating {
        clauses.push("rating >= ?".to_string());
        params.push(Value::Integer(min));
    }
    if let Some(max) = query.max_rating {
        clauses.push("rating <= ?".to_string());
        params.push(Value::Integer(max));
    }

    if clauses.is_empty() {
        (String::new(), params)
    } else {
        (format!(" WHERE {}", clauses.join(" AND ")), params)
    }
}

/// Push a list of text values as parameters and return the `?, ?, …` placeholder
/// string for them.
fn push_text_list<'s>(
    params: &mut Vec<Value>,
    values: impl Iterator<Item = &'s str>,
) -> String {
    let start = params.len();
    for value in values {
        params.push(Value::Text(value.to_string()));
    }
    vec!["?"; params.len() - start].join(", ")
}

/// Map sort field + direction to a stable `ORDER BY` clause (NULLs last, tie-broken by id).
fn order_clause(sort: GameSort, direction: SortDirection) -> String {
    // The default ordering is a fixed composite that ignores direction:
    // currently-playing games first, then most recently finished.
    if sort == GameSort::Default {
        return "CASE status WHEN 'playing' THEN 0 ELSE 1 END, \
                finished_at DESC NULLS LAST, title COLLATE NOCASE ASC, id DESC"
            .to_string();
    }

    let column = match sort {
        GameSort::Title => "title COLLATE NOCASE",
        GameSort::StartedAt => "started_at",
        GameSort::FinishedAt => "finished_at",
        GameSort::ReleaseYear => "release_year",
        GameSort::Rating => "rating",
        GameSort::Status => "status",
        GameSort::Default => unreachable!("handled above"),
    };
    let dir = match direction {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };
    format!("{column} {dir} NULLS LAST, id {dir}")
}

// Bridge the domain enum to SQLite. Kept in the repository layer so the domain
// model stays free of any persistence dependency.
impl ToSql for GameStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for GameStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        GameStatus::parse(value.as_str()?).ok_or(FromSqlError::InvalidType)
    }
}
