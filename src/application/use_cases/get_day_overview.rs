/// GetDayOverview use case

use crate::application::dto::{GetDayOverviewInput, DayOverview, SuggestedSlot};
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::{UserRepository, TaskRepository, ScheduleRepository};
use crate::application::types::UserId;
use crate::domain::entities::schedule::{expand_template, find_candidate_slots, TimeBlock};
use chrono::Duration;

/// Use case for getting a day overview with schedule and task suggestions
pub struct GetDayOverview<'a> {
    user_repo: &'a dyn UserRepository,
    task_repo: &'a dyn TaskRepository,
    schedule_repo: &'a dyn ScheduleRepository,
}

impl<'a> GetDayOverview<'a> {
    pub fn new(
        user_repo: &'a dyn UserRepository,
        task_repo: &'a dyn TaskRepository,
        schedule_repo: &'a dyn ScheduleRepository,
    ) -> Self {
        Self {
            user_repo,
            task_repo,
            schedule_repo,
        }
    }

    pub fn execute(&self, user_id: UserId, input: GetDayOverviewInput) -> AppResult<DayOverview> {
        // Get the user to access their location and week_start
        let user = self.user_repo.find_by_id(user_id)?;

        // Get the user's active schedule template
        let active_template_id = self.user_repo.get_active_schedule_template(user_id)?
            .ok_or_else(|| AppError::ValidationError("User has no active schedule template".to_string()))?;

        let template = self.schedule_repo.find_template(user_id, active_template_id)?;

        // Expand the template for the requested day
        let start_of_day = input.date;
        let end_of_day = input.date + Duration::days(1);
        
        let time_blocks = expand_template(
            &template,
            start_of_day,
            end_of_day,
        );

        // Get active tasks for the day
        let tasks = self.task_repo.find_tasks_for_date(user_id, input.date.with_timezone(&chrono::Utc))?;

        // For now, we don't have scheduled tasks (that would require a separate occurrence tracking system)
        let scheduled_tasks = Vec::new();

        // Generate suggestions for each task
        let mut suggestions = Vec::new();
        
        // Get user's current location (take the first known location)
        let user_location = user.locations.iter()
            .find(|loc| loc.is_some())
            .and_then(|loc| loc.clone());

        for (task_id, task) in tasks {
            // Find candidate slots where this task could be scheduled
            let candidate_times: Vec<(chrono::DateTime<chrono::FixedOffset>, chrono::DateTime<chrono::FixedOffset>)> = 
                find_candidate_slots(
                    &time_blocks,
                    &task,
                    user_location.as_ref(),
                );

            // Convert to SuggestedSlot DTOs with scoring
            // For MVP, we use a simple scoring: higher priority = higher score
            let task_suggestions: Vec<SuggestedSlot> = candidate_times
                .into_iter()
                .take(5) // Limit to 5 suggestions per task (techno-business rule)
                .enumerate()
                .map(|(idx, (start, end))| {
                    // Simple scoring: first slots get higher scores
                    let score = 100 - (idx as u8 * 10).min(50);
                    
                    let reason = format!(
                        "Available slot at {}",
                        start.format("%H:%M")
                    );

                    // Find the corresponding TimeBlock for this candidate
                    // (In a more sophisticated implementation, we'd track this better)
                    let time_block = time_blocks.iter()
                        .find(|block| block.start <= start && block.end >= end)
                        .cloned()
                        .unwrap_or_else(|| {
                            // Fallback: create a minimal TimeBlock
                            // This shouldn't happen, but we handle it gracefully
                            use crate::domain::entities::schedule::{AvailabilityKind, CapabilitySet, LocationConstraint};
                            TimeBlock {
                                start,
                                end,
                                availability: AvailabilityKind::Available,
                                capabilities: CapabilitySet::free(),
                                location_constraint: LocationConstraint::Any,
                                label: None,
                                priority: 0,
                            }
                        });

                    SuggestedSlot {
                        time_block,
                        score,
                        reason,
                    }
                })
                .collect();

            if !task_suggestions.is_empty() {
                suggestions.push((task_id, task_suggestions));
            }
        }

        Ok(DayOverview {
            date: input.date,
            time_blocks,
            scheduled_tasks,
            suggestions,
        })
    }
}
