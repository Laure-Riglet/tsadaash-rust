/// SetActiveScheduleTemplate use case

use crate::application::errors::AppResult;
use crate::application::ports::{ScheduleRepository, UserRepository};
use crate::application::types::{UserId, ScheduleTemplateId};

/// Use case for setting the active schedule template for a user
pub struct SetActiveScheduleTemplate<'a> {
    user_repo: &'a mut dyn UserRepository,
    schedule_repo: &'a dyn ScheduleRepository,
}

impl<'a> SetActiveScheduleTemplate<'a> {
    pub fn new(
        user_repo: &'a mut dyn UserRepository,
        schedule_repo: &'a dyn ScheduleRepository,
    ) -> Self {
        Self {
            user_repo,
            schedule_repo,
        }
    }

    pub fn execute(&mut self, user_id: UserId, template_id: Option<ScheduleTemplateId>) -> AppResult<()> {
        // If a template ID is provided, verify it exists and belongs to the user
        if let Some(tid) = template_id {
            let _ = self.schedule_repo.find_template(user_id, tid)?;
        }

        // Set the active template
        self.user_repo.set_active_schedule_template(user_id, template_id)?;

        Ok(())
    }
}
