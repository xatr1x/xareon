use crate::domain::achievement::{
    Achievement, AchievementStatus, AchievementUpdate, NewAchievement,
};
use crate::error::{AppError, AppResult};
use crate::repositories::achievement_repository::AchievementRepository;
use crate::repositories::game_repository::GameRepository;
use crate::validation;

pub struct AchievementService<'a, AR: AchievementRepository, GR: GameRepository> {
    achievements: &'a AR,
    games: &'a GR,
}

impl<'a, AR: AchievementRepository, GR: GameRepository> AchievementService<'a, AR, GR> {
    pub fn new(achievements: &'a AR, games: &'a GR) -> Self {
        Self {
            achievements,
            games,
        }
    }

    pub fn list_for_game(&self, game_id: i64) -> AppResult<Vec<Achievement>> {
        self.achievements.list_for_game(game_id)
    }

    pub fn create(&self, mut input: NewAchievement) -> AppResult<Achievement> {
        self.games.get(input.game_id)?;
        normalize_new(&mut input);
        validate(
            &input.title,
            input.progress_current,
            input.progress_target,
            input.display_order.unwrap_or(0),
        )?;
        apply_progress_status(&mut input.status, input.progress_current, input.progress_target);

        let id = self.achievements.create(&input)?;
        self.achievements.get(id)
    }

    pub fn update(&self, id: i64, mut update: AchievementUpdate) -> AppResult<Achievement> {
        normalize_update(&mut update);
        validate(
            &update.title,
            update.progress_current,
            update.progress_target,
            update.display_order,
        )?;
        apply_progress_status(
            &mut update.status,
            update.progress_current,
            update.progress_target,
        );

        self.achievements.update(id, &update)?;
        self.achievements.get(id)
    }

    pub fn set_progress(&self, id: i64, progress_current: i64) -> AppResult<Achievement> {
        if progress_current < 0 {
            return Err(AppError::Validation(
                "progressCurrent must be at least 0".to_string(),
            ));
        }

        let achievement = self.achievements.get(id)?;
        let mut update = AchievementUpdate {
            title: achievement.title,
            description: achievement.description,
            category: achievement.category,
            status: achievement.status,
            progress_current: Some(progress_current),
            progress_target: achievement.progress_target,
            progress_unit: achievement.progress_unit,
            completed_at: achievement.completed_at,
            is_hidden: achievement.is_hidden,
            display_order: achievement.display_order,
        };
        validate(
            &update.title,
            update.progress_current,
            update.progress_target,
            update.display_order,
        )?;
        apply_progress_status(
            &mut update.status,
            update.progress_current,
            update.progress_target,
        );

        self.achievements.update(id, &update)?;
        self.achievements.get(id)
    }

    pub fn complete(&self, id: i64) -> AppResult<Achievement> {
        let achievement = self.achievements.get(id)?;
        let mut update = AchievementUpdate {
            title: achievement.title,
            description: achievement.description,
            category: achievement.category,
            status: AchievementStatus::Completed,
            progress_current: achievement.progress_target.or(achievement.progress_current),
            progress_target: achievement.progress_target,
            progress_unit: achievement.progress_unit,
            completed_at: achievement.completed_at,
            is_hidden: achievement.is_hidden,
            display_order: achievement.display_order,
        };
        normalize_update(&mut update);
        self.achievements.update(id, &update)?;
        self.achievements.get(id)
    }

    pub fn reopen(&self, id: i64) -> AppResult<Achievement> {
        let achievement = self.achievements.get(id)?;
        let update = AchievementUpdate {
            title: achievement.title,
            description: achievement.description,
            category: achievement.category,
            status: AchievementStatus::InProgress,
            progress_current: achievement.progress_current,
            progress_target: achievement.progress_target,
            progress_unit: achievement.progress_unit,
            completed_at: None,
            is_hidden: achievement.is_hidden,
            display_order: achievement.display_order,
        };
        self.achievements.update(id, &update)?;
        self.achievements.get(id)
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        self.achievements.delete(id)
    }
}

fn normalize_new(input: &mut NewAchievement) {
    input.title = input.title.trim().to_string();
    input.description = clean_optional(input.description.take());
    input.category = clean_optional(input.category.take());
    input.progress_unit = clean_optional(input.progress_unit.take());
    input.completed_at = clean_optional(input.completed_at.take());
    if input.progress_target.is_some() && input.progress_current.is_none() {
        input.progress_current = Some(0);
    }
}

fn normalize_update(update: &mut AchievementUpdate) {
    update.title = update.title.trim().to_string();
    update.description = clean_optional(update.description.take());
    update.category = clean_optional(update.category.take());
    update.progress_unit = clean_optional(update.progress_unit.take());
    update.completed_at = clean_optional(update.completed_at.take());
    if update.progress_target.is_some() && update.progress_current.is_none() {
        update.progress_current = Some(0);
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn validate(
    title: &str,
    progress_current: Option<i64>,
    progress_target: Option<i64>,
    display_order: i64,
) -> AppResult<()> {
    validation::require_non_empty("title", title)?;
    if display_order < 0 {
        return Err(AppError::Validation(
            "displayOrder must be at least 0".to_string(),
        ));
    }
    if let Some(current) = progress_current {
        if current < 0 {
            return Err(AppError::Validation(
                "progressCurrent must be at least 0".to_string(),
            ));
        }
    }
    if let Some(target) = progress_target {
        if target <= 0 {
            return Err(AppError::Validation(
                "progressTarget must be greater than 0".to_string(),
            ));
        }
    }
    if let (Some(current), Some(target)) = (progress_current, progress_target) {
        if current > target {
            return Err(AppError::Validation(
                "progressCurrent must not exceed progressTarget".to_string(),
            ));
        }
    }
    Ok(())
}

fn apply_progress_status(
    status: &mut AchievementStatus,
    progress_current: Option<i64>,
    progress_target: Option<i64>,
) {
    if let (Some(current), Some(target)) = (progress_current, progress_target) {
        if current >= target {
            *status = AchievementStatus::Completed;
        }
    }
}
