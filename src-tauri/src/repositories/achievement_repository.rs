use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, OptionalExtension, Row};

use crate::domain::achievement::{
    Achievement, AchievementStatus, AchievementUpdate, NewAchievement,
};
use crate::error::{AppError, AppResult};

pub trait AchievementRepository {
    fn list_for_game(&self, game_id: i64) -> AppResult<Vec<Achievement>>;
    fn get(&self, id: i64) -> AppResult<Achievement>;
    fn create(&self, achievement: &NewAchievement) -> AppResult<i64>;
    fn update(&self, id: i64, update: &AchievementUpdate) -> AppResult<()>;
    fn delete(&self, id: i64) -> AppResult<()>;
}

const COLUMNS: &str = "id, game_id, title, description, category, status, progress_current, \
    progress_target, progress_unit, completed_at, is_hidden, display_order, created_at, updated_at";

pub struct SqliteAchievementRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteAchievementRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn map_row(row: &Row<'_>) -> rusqlite::Result<Achievement> {
        Ok(Achievement {
            id: row.get("id")?,
            game_id: row.get("game_id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            category: row.get("category")?,
            status: row.get("status")?,
            progress_current: row.get("progress_current")?,
            progress_target: row.get("progress_target")?,
            progress_unit: row.get("progress_unit")?,
            completed_at: row.get("completed_at")?,
            is_hidden: row.get("is_hidden")?,
            display_order: row.get("display_order")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

impl AchievementRepository for SqliteAchievementRepository<'_> {
    fn list_for_game(&self, game_id: i64) -> AppResult<Vec<Achievement>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM achievements WHERE game_id = ?1 \
             ORDER BY display_order ASC, category IS NULL ASC, category COLLATE NOCASE ASC, \
             created_at ASC, id ASC"
        ))?;
        let achievements = stmt
            .query_map([game_id], Self::map_row)?
            .collect::<rusqlite::Result<Vec<Achievement>>>()?;
        Ok(achievements)
    }

    fn get(&self, id: i64) -> AppResult<Achievement> {
        self.conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM achievements WHERE id = ?1"),
                [id],
                Self::map_row,
            )
            .optional()?
            .ok_or(AppError::NotFound)
    }

    fn create(&self, achievement: &NewAchievement) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO achievements \
             (game_id, title, description, category, status, progress_current, progress_target, \
              progress_unit, completed_at, is_hidden, display_order) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, \
              CASE WHEN ?5 = 'completed' THEN COALESCE(?9, datetime('now')) ELSE NULL END, \
              ?10, ?11)",
            rusqlite::params![
                achievement.game_id,
                achievement.title,
                achievement.description,
                achievement.category,
                achievement.status,
                achievement.progress_current,
                achievement.progress_target,
                achievement.progress_unit,
                achievement.completed_at,
                achievement.is_hidden,
                achievement.display_order.unwrap_or(0),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    fn update(&self, id: i64, update: &AchievementUpdate) -> AppResult<()> {
        let affected = self.conn.execute(
            "UPDATE achievements SET \
             title = ?1, description = ?2, category = ?3, status = ?4, \
             progress_current = ?5, progress_target = ?6, progress_unit = ?7, \
             completed_at = CASE \
                 WHEN ?4 = 'completed' THEN COALESCE(?8, completed_at, datetime('now')) \
                 ELSE NULL \
             END, \
             is_hidden = ?9, display_order = ?10, updated_at = datetime('now') \
             WHERE id = ?11",
            rusqlite::params![
                update.title,
                update.description,
                update.category,
                update.status,
                update.progress_current,
                update.progress_target,
                update.progress_unit,
                update.completed_at,
                update.is_hidden,
                update.display_order,
                id,
            ],
        )?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    fn delete(&self, id: i64) -> AppResult<()> {
        let affected = self
            .conn
            .execute("DELETE FROM achievements WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}

impl ToSql for AchievementStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for AchievementStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let value = value.as_str()?;
        AchievementStatus::parse(value)
            .ok_or_else(|| FromSqlError::Other("invalid achievement status".into()))
    }
}
