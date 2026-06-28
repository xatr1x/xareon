use crate::domain::game::{Game, GameInput, GameQuery};
use crate::error::AppResult;
use crate::repositories::game_repository::GameRepository;
use crate::repositories::genre_repository::GenreRepository;
use crate::validation;

/// Coordinates game operations and enforces business rules. Orchestrates the
/// game and genre repositories: a game's genres are part of the same logical
/// operation, so writes must run inside a transaction (set up by the caller).
pub struct GameService<'a, GR: GameRepository, GnR: GenreRepository> {
    games: &'a GR,
    genres: &'a GnR,
}

impl<'a, GR: GameRepository, GnR: GenreRepository> GameService<'a, GR, GnR> {
    pub fn new(games: &'a GR, genres: &'a GnR) -> Self {
        Self { games, genres }
    }

    pub fn query(&self, query: &GameQuery) -> AppResult<Vec<Game>> {
        self.games.query(query)
    }

    pub fn get(&self, id: i64) -> AppResult<Game> {
        self.games.get(id)
    }

    pub fn create(&self, input: GameInput) -> AppResult<Game> {
        Self::validate(&input)?;
        let id = self.games.create(&input)?;
        self.genres.replace_for_game(id, &input.genres)?;
        self.games.get(id)
    }

    pub fn update(&self, id: i64, input: GameInput) -> AppResult<Game> {
        Self::validate(&input)?;
        self.games.update(id, &input)?;
        self.genres.replace_for_game(id, &input.genres)?;
        self.games.get(id)
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        // `game_genres` and `journal_entries` cascade via foreign keys.
        self.games.delete(id)
    }

    fn validate(input: &GameInput) -> AppResult<()> {
        validation::require_non_empty("title", &input.title)?;
        validation::require_in_range("rating", input.rating, 0, 10)?;
        Ok(())
    }
}
