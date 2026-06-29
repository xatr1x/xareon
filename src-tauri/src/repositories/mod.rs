//! Repositories — the only place that knows SQL. Services depend on the traits
//! here, not on the concrete SQLite implementations.

pub mod game_repository;
pub mod genre_repository;
pub mod journal_repository;
pub mod settings_repository;