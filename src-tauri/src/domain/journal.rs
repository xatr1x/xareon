use serde::{Deserialize, Serialize};

/// A journal entry — a first-class record of the player's thoughts during a
/// playthrough, owned by a game. Designed to grow: future metadata (mood, tags,
/// screenshots) can be added without changing the entry's identity.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub id: i64,
    pub game_id: i64,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Fields for creating a journal entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewJournalEntry {
    pub game_id: i64,
    pub body: String,
}

/// Fields for editing a journal entry. The owning game never changes.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryUpdate {
    pub body: String,
}
