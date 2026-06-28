//! Database access: the `DatabaseManager` gateway, opening a connection and
//! applying versioned migrations.

pub mod connection;
pub mod manager;
pub mod migrations;
