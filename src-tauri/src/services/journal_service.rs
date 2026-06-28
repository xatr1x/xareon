use crate::domain::journal::{JournalEntry, JournalEntryUpdate, NewJournalEntry};
use crate::error::AppResult;
use crate::repositories::game_repository::GameRepository;
use crate::repositories::journal_repository::JournalRepository;
use crate::validation;

/// Business rules for journal entries. Depends on the game repository to ensure
/// an entry always belongs to an existing game.
pub struct JournalService<'a, JR: JournalRepository, GR: GameRepository> {
    journal: &'a JR,
    games: &'a GR,
}

impl<'a, JR: JournalRepository, GR: GameRepository> JournalService<'a, JR, GR> {
    pub fn new(journal: &'a JR, games: &'a GR) -> Self {
        Self { journal, games }
    }

    pub fn list_for_game(&self, game_id: i64) -> AppResult<Vec<JournalEntry>> {
        self.journal.list_for_game(game_id)
    }

    pub fn create(&self, entry: NewJournalEntry) -> AppResult<JournalEntry> {
        validation::require_non_empty("entry", &entry.body)?;
        // Surface a clean NotFound instead of a foreign-key failure.
        self.games.get(entry.game_id)?;
        let id = self.journal.create(&entry)?;
        self.journal.get(id)
    }

    pub fn update(&self, id: i64, update: JournalEntryUpdate) -> AppResult<JournalEntry> {
        validation::require_non_empty("entry", &update.body)?;
        self.journal.update_body(id, &update.body)?;
        self.journal.get(id)
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        self.journal.delete(id)
    }
}
