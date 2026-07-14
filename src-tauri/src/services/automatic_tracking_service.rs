use crate::domain::automatic_tracking::ExecutableBinding;
use crate::error::AppResult;
use crate::repositories::game_process_repository::GameProcessRepository;

pub struct AutomaticTrackingService<'a, R: GameProcessRepository> { processes: &'a R }

impl<'a, R: GameProcessRepository> AutomaticTrackingService<'a, R> {
    pub fn new(processes: &'a R) -> Self { Self { processes } }
    pub fn bindings(&self, game_id: i64) -> AppResult<Vec<ExecutableBinding>> { self.processes.list_for_game(game_id) }
    pub fn add(&self, game_id: i64, path: &str) -> AppResult<ExecutableBinding> { self.processes.add(game_id, path) }
    pub fn delete(&self, game_id: i64, binding_id: i64) -> AppResult<()> { self.processes.delete(game_id, binding_id) }
    pub fn set_enabled(&self, game_id: i64, enabled: bool) -> AppResult<()> { self.processes.set_enabled(game_id, enabled) }
    pub fn is_enabled(&self, game_id: i64) -> AppResult<bool> { self.processes.is_enabled(game_id) }
}
