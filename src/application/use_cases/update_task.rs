/// UpdateTask use case

use crate::application::dto::UpdateTaskInput;
use crate::application::errors::AppResult;
use crate::application::ports::TaskRepository;
use crate::application::types::{UserId, TaskId};

/// Use case for updating an existing task
pub struct UpdateTask<'a> {
    task_repo: &'a mut dyn TaskRepository,
}

impl<'a> UpdateTask<'a> {
    pub fn new(task_repo: &'a mut dyn TaskRepository) -> Self {
        Self { task_repo }
    }

    pub fn execute(&mut self, user_id: UserId, task_id: TaskId, input: UpdateTaskInput) -> AppResult<()> {
        // Load the existing task
        let mut task = self.task_repo.find_by_id(user_id, task_id)?;

        // Update fields if provided
        if let Some(title) = input.title {
            task.set_title(title)
                .map_err(|e| crate::application::errors::AppError::ValidationError(e.to_string()))?;
        }

        if let Some(description) = input.description {
            task.set_description(description)
                .map_err(|e| crate::application::errors::AppError::ValidationError(e.to_string()))?;
        }

        if let Some(priority) = input.priority {
            task.set_priority(priority);
        }

        if let Some(periodicity) = input.periodicity {
            task.set_periodicity(periodicity);
        }

        // Update scheduling attributes if provided
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
        if let Some(locations) = input.locations {
            task.set_locations(locations);
        }

        // Save the updated task
        self.task_repo.update(user_id, task_id, task)?;

        Ok(())
    }
}
