use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::domain::game::{Game, GameInput, GameStatus};
use crate::error::{AppError, AppResult};

/// Abstract persistence operations for games. Higher layers depend on this trait
/// so the storage engine can change without touching services or commands.
pub trait GameRepository {
    fn list(&self) -> AppResult<Vec<Game>>;
    fn get(&self, id: i64) -> AppResult<Game>;
    fn create(&self, input: &GameInput) -> AppResult<Game>;
    fn update(&self, id: i64, input: &GameInput) -> AppResult<Game>;
    fn delete(&self, id: i64) -> AppResult<()>;
}

/// Columns selected when reading a full game row, in `map_row` order.
const COLUMNS: &str = "id, title, genre, platform, developer, publisher, \
    release_year, started_at, finished_at, status, rating, cover_path, \
    created_at, updated_at";

/// SQLite-backed implementation borrowing a connection for the duration of a call.
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
            genre: row.get("genre")?,
            platform: row.get("platform")?,
            developer: row.get("developer")?,
            publisher: row.get("publisher")?,
            release_year: row.get("release_year")?,
            started_at: row.get("started_at")?,
            finished_at: row.get("finished_at")?,
            status: row.get("status")?,
            rating: row.get("rating")?,
            cover_path: row.get("cover_path")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

impl GameRepository for SqliteGameRepository<'_> {
    fn list(&self) -> AppResult<Vec<Game>> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT {COLUMNS} FROM games ORDER BY title COLLATE NOCASE"))?;
        let games = stmt
            .query_map([], Self::map_row)?
            .collect::<rusqlite::Result<Vec<Game>>>()?;
        Ok(games)
    }

    fn get(&self, id: i64) -> AppResult<Game> {
        self.conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM games WHERE id = ?1"),
                [id],
                Self::map_row,
            )
            .optional()?
            .ok_or(AppError::NotFound)
    }

    fn create(&self, input: &GameInput) -> AppResult<Game> {
        self.conn.execute(
            "INSERT INTO games \
             (title, genre, platform, developer, publisher, release_year, \
              started_at, finished_at, status, rating, cover_path) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                input.title,
                input.genre,
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
        self.get(self.conn.last_insert_rowid())
    }

    fn update(&self, id: i64, input: &GameInput) -> AppResult<Game> {
        let affected = self.conn.execute(
            "UPDATE games SET \
             title = ?1, genre = ?2, platform = ?3, developer = ?4, publisher = ?5, \
             release_year = ?6, started_at = ?7, finished_at = ?8, status = ?9, \
             rating = ?10, cover_path = ?11, updated_at = datetime('now') \
             WHERE id = ?12",
            params![
                input.title,
                input.genre,
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
        self.get(id)
    }

    fn delete(&self, id: i64) -> AppResult<()> {
        let affected = self.conn.execute("DELETE FROM games WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
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
