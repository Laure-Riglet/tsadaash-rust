/// CreateScheduleTemplate use case

use crate::application::dto::{CreateScheduleTemplateInput, CreateScheduleTemplateOutput};
use crate::application::errors::AppResult;
use crate::application::ports::ScheduleRepository;
use crate::application::types::UserId;
use crate::domain::entities::schedule::ScheduleTemplate;

/// Use case for creating a new schedule template
pub struct CreateScheduleTemplate<'a> {
    schedule_repo: &'a mut dyn ScheduleRepository,
}

impl<'a> CreateScheduleTemplate<'a> {
    pub fn new(schedule_repo: &'a mut dyn ScheduleRepository) -> Self {
        Self { schedule_repo }
    }

    pub fn execute(&mut self, user_id: UserId, input: CreateScheduleTemplateInput) -> AppResult<CreateScheduleTemplateOutput> {
        // Create the domain entity (no persistence IDs at domain level)
        let template = ScheduleTemplate::new(
            input.name.clone(),
            input.description.unwrap_or_else(|| "UTC".to_string()), // Use description as timezone for now, or default to UTC
            Vec::new(), // Start with no rules
        )?;

        // Save the template
        let template_id = self.schedule_repo.save_template(user_id, template)?;

        Ok(CreateScheduleTemplateOutput {
            template_id,
            name: input.name,
        })
    }
}
