/// UpdateUserSettings use case

use crate::application::dto::UpdateUserSettingsInput;
use crate::application::errors::AppResult;
use crate::application::ports::UserRepository;
use crate::application::types::UserId;

/// Use case for updating user settings
pub struct UpdateUserSettings<'a> {
    user_repo: &'a mut dyn UserRepository,
}

impl<'a> UpdateUserSettings<'a> {
    pub fn new(user_repo: &'a mut dyn UserRepository) -> Self {
        Self { user_repo }
    }

    pub fn execute(&mut self, user_id: UserId, input: UpdateUserSettingsInput) -> AppResult<()> {
        // Load the user
        let mut user = self.user_repo.find_by_id(user_id)?;

        // Update fields if provided
        if let Some(week_start) = input.week_start {
            user.set_week_start(week_start);
        }

        if let Some(year_start) = input.year_start {
            user.set_year_start(year_start);
        }

        if let Some(day_start) = input.day_start {
            user.set_day_start(day_start);
        }

        if let Some(timezone) = input.timezone {
            user.timezone = timezone;
        }

        // Save the updated user
        self.user_repo.update(user_id, user)?;

        Ok(())
    }
}
