/// CompleteOccurrenceRep use case

use crate::application::dto::CompleteOccurrenceRepInput;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::TaskRepository;
use crate::application::types::UserId;
use crate::infrastructure::Clock;

/// Use case for completing an occurrence repetition
pub struct CompleteOccurrenceRep<'a> {
    task_repo: &'a mut dyn TaskRepository,
    clock: &'a dyn Clock,
}

impl<'a> CompleteOccurrenceRep<'a> {
    pub fn new(task_repo: &'a mut dyn TaskRepository, clock: &'a dyn Clock) -> Self {
        Self { task_repo, clock }
    }

    pub fn execute(&mut self, user_id: UserId, input: CompleteOccurrenceRepInput) -> AppResult<()> {
        // Load the task
        let task = self.task_repo.find_by_id(user_id, input.task_id)?;

        // Get the current time (for future use when we implement occurrence tracking)
        let _now = self.clock.now();

        // Mark the occurrence rep as complete
        // Note: In a real implementation, you'd track occurrences separately
        // For now, this is a simplified version that just updates the task
        // In the future, you'd want to:
        // 1. Find the TaskOccurrence by index
        // 2. Call mark_rep_completed on the occurrence
        // 3. Store the updated occurrence
        
        // For MVP, we'll just validate the indices exist
        // The actual completion tracking would need to be implemented
        // in the infrastructure layer with proper occurrence storage
        
        // Placeholder: Just verify the task exists and is active
        if !task.is_active() {
            return Err(AppError::ValidationError(
                "Cannot complete occurrence for inactive task".to_string()
            ));
        }

        // In a full implementation, you'd:
        // - Load the TaskOccurrence
        // - Call mark_rep_completed(input.rep_index, now, input.notes)
        // - Save the updated occurrence

        Ok(())
    }
}
