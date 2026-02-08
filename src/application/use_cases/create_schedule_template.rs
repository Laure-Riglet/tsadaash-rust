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
        // Note: In a clean architecture, the domain shouldn't know about IDs
        // For MVP, we pass placeholder values that will be overridden by the repository
        // TODO: Consider refactoring ScheduleTemplate to not require ID in constructor
        
        let template = ScheduleTemplate::new(
            0, // Placeholder ID - repository will assign actual ID
            user_id.value() as i32, // Convert UserId to i32
            input.name.clone(),
            input.description.unwrap_or_else(|| "UTC".to_string()), // Use description as timezone placeholder or UTC as default
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
