use crate::domain::play_session::{PlaySession, PlayTimeTotals};
use crate::error::AppResult;
use crate::repositories::play_session_repository::PlaySessionRepository;

pub struct PlaySessionService<'a, R: PlaySessionRepository> { sessions: &'a R }

impl<'a, R: PlaySessionRepository> PlaySessionService<'a, R> {
    pub fn new(sessions: &'a R) -> Self { Self { sessions } }
    pub fn active(&self) -> AppResult<Option<PlaySession>> { self.sessions.active() }
    pub fn start(&self, game_id: i64) -> AppResult<PlaySession> { self.sessions.start(game_id) }
    pub fn heartbeat(&self, game_id: i64) -> AppResult<PlaySession> { self.sessions.heartbeat(game_id) }
    pub fn stop(&self, game_id: i64) -> AppResult<()> { self.sessions.stop(game_id) }
    pub fn totals(&self) -> AppResult<PlayTimeTotals> { self.sessions.play_time_totals() }
    pub fn game_today(&self, game_id: i64) -> AppResult<i64> { self.sessions.game_seconds_today(game_id) }
    pub fn history(&self, game_id: i64) -> AppResult<Vec<PlaySession>> { self.sessions.list_for_game(game_id, 20) }
}
