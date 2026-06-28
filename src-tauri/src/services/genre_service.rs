use crate::domain::genre::Genre;
use crate::error::AppResult;
use crate::repositories::genre_repository::GenreRepository;

/// Read access to genres for the UI (e.g. input suggestions and, later, genre
/// management and statistics).
pub struct GenreService<'a, R: GenreRepository> {
    genres: &'a R,
}

impl<'a, R: GenreRepository> GenreService<'a, R> {
    pub fn new(genres: &'a R) -> Self {
        Self { genres }
    }

    pub fn list(&self) -> AppResult<Vec<Genre>> {
        self.genres.list_all()
    }
}
