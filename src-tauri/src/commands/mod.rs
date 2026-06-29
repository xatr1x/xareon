//! Tauri command handlers — the thin boundary between the frontend and the
//! application services. They wire up repositories + a service per call (writes
//! inside a transaction) and hold no business logic of their own.

pub mod game_commands;
pub mod genre_commands;
pub mod journal_commands;
pub mod settings_commands;