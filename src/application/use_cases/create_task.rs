/// CreateTask use case

use crate::application::dto::{CreateTaskInput, CreateTaskOutput};
use crate::application::errors::AppResult;
use crate::application::ports::TaskRepository;
use crate::application::types::UserId;
use crate::domain::entities::task::Task;

/// Use case for creating a new task
pub struct CreateTask<'a> {
    task_repo: &'a mut dyn TaskRepository,
}

impl<'a> CreateTask<'a> {
    pub fn new(task_repo: &'a mut dyn TaskRepository) -> Self {
        Self { task_repo }
    }

    pub fn execute(&mut self, user_id: UserId, input: CreateTaskInput) -> AppResult<CreateTaskOutput> {
        // Create the task with domain validation
        let mut task = Task::new(
            input.title.clone(),
            input.periodicity,
        )
        .map_err(|e| crate::application::errors::AppError::ValidationError(e.to_string()))?;

        // Set optional fields
        if let Some(description) = input.description {
            task.set_description(Some(description))
                .map_err(|e| crate::application::errors::AppError::ValidationError(e.to_string()))?;
        }

        if let Some(priority) = input.priority {
            task.set_priority(priority);
        }

        // Apply scheduling attributes if provided
        if let Some(min_hands) = input.min_hands {
            task.set_min_hands(min_hands);
        }
        if let Some(min_eyes) = input.min_eyes {
            task.set_min_eyes(min_eyes);
        }
        if let Some(min_speech) = input.min_speech {
            task.set_min_speech(min_speech);
        }
        if let Some(min_cognitive) = input.min_cognitive {
            task.set_min_cognitive(min_cognitive);
        }
        if let Some(min_device) = input.min_device {
            task.set_min_device(min_device);
        }
        if let Some(allowed_mobility) = input.allowed_mobility {
            task.set_allowed_mobility(vec![allowed_mobility]);
        }
        if !input.locations.is_empty() {
            task.set_locations(input.locations);
        }

        // Save the task
        let task_id = self.task_repo.save(user_id, task)?;

        Ok(CreateTaskOutput {
            task_id,
            title: input.title,
        })
    }
}
