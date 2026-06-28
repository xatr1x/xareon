use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::genre::{normalize, Genre};
use crate::error::AppResult;

/// Owns the `genres` table and writes to the `game_genres` link table.
/// Recognition/deduplication is by normalized name.
pub trait GenreRepository {
    /// Return the genre with this (normalized) name, creating it if absent.
    fn get_or_create(&self, name: &str) -> AppResult<Genre>;
    /// Replace a game's genres with exactly this set of names (resolved/created).
    fn replace_for_game(&self, game_id: i64, names: &[String]) -> AppResult<()>;
    /// All genres, alphabetical — for management and input suggestions.
    fn list_all(&self) -> AppResult<Vec<Genre>>;
}

pub struct SqliteGenreRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteGenreRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl GenreRepository for SqliteGenreRepository<'_> {
    fn get_or_create(&self, name: &str) -> AppResult<Genre> {
        let display = name.trim();
        let normalized = normalize(name);

        // Insert if new; ignore if it already exists.
        self.conn.execute(
            "INSERT INTO genres (name, name_normalized) VALUES (?1, ?2) \
             ON CONFLICT(name_normalized) DO NOTHING",
            params![display, normalized],
        )?;

        let genre = self
            .conn
            .query_row(
                "SELECT id, name FROM genres WHERE name_normalized = ?1",
                [&normalized],
                |row| {
                    Ok(Genre {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                },
            )
            .optional()?;

        // The row was just inserted or already existed, so this is always Some.
        genre.ok_or_else(|| {
            crate::error::AppError::Validation(format!("could not resolve genre '{display}'"))
        })
    }

    fn replace_for_game(&self, game_id: i64, names: &[String]) -> AppResult<()> {
        self.conn
            .execute("DELETE FROM game_genres WHERE game_id = ?1", [game_id])?;

        // Deduplicate by normalized name to avoid inserting the same link twice.
        let mut seen = std::collections::HashSet::new();
        for name in names {
            let normalized = normalize(name);
            if normalized.is_empty() || !seen.insert(normalized) {
                continue;
            }
            let genre = self.get_or_create(name)?;
            self.conn.execute(
                "INSERT INTO game_genres (game_id, genre_id) VALUES (?1, ?2)",
                params![game_id, genre.id],
            )?;
        }
        Ok(())
    }

    fn list_all(&self) -> AppResult<Vec<Genre>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM genres ORDER BY name COLLATE NOCASE")?;
        let genres = stmt
            .query_map([], |row| {
                Ok(Genre {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<Genre>>>()?;
        Ok(genres)
    }
}
