use crate::domain::game::{Game, GameInput};
use crate::error::AppResult;
use crate::repositories::game_repository::GameRepository;
use crate::validation;

/// Coordinates game operations and enforces business rules before delegating to
/// the repository. Generic over the repository so it can be unit-tested with a
/// fake implementation.
pub struct GameService<'a, R: GameRepository> {
    repo: &'a R,
}

impl<'a, R: GameRepository> GameService<'a, R> {
    pub fn new(repo: &'a R) -> Self {
        Self { repo }
    }

    pub fn list(&self) -> AppResult<Vec<Game>> {
        self.repo.list()
    }

    pub fn get(&self, id: i64) -> AppResult<Game> {
        self.repo.get(id)
    }

    pub fn create(&self, input: GameInput) -> AppResult<Game> {
        Self::validate(&input)?;
        self.repo.create(&input)
    }

    pub fn update(&self, id: i64, input: GameInput) -> AppResult<Game> {
        Self::validate(&input)?;
        self.repo.update(id, &input)
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        self.repo.delete(id)
    }

    fn validate(input: &GameInput) -> AppResult<()> {
        validation::require_non_empty("title", &input.title)?;
        validation::require_in_range("rating", input.rating, 0, 10)?;
        Ok(())
    }
}
