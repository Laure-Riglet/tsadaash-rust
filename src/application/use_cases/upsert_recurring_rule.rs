/// UpsertRecurringRule use case

use crate::application::dto::{UpsertRecurringRuleInput, UpsertRecurringRuleOutput};
use crate::application::errors::AppResult;
use crate::application::ports::ScheduleRepository;
use crate::application::types::UserId;
use crate::domain::entities::schedule::RecurringRule;

/// Use case for upserting a recurring rule in a schedule template
pub struct UpsertRecurringRule<'a> {
    schedule_repo: &'a mut dyn ScheduleRepository,
}

impl<'a> UpsertRecurringRule<'a> {
    pub fn new(schedule_repo: &'a mut dyn ScheduleRepository) -> Self {
        Self { schedule_repo }
    }

    pub fn execute(&mut self, user_id: UserId, input: UpsertRecurringRuleInput) -> AppResult<UpsertRecurringRuleOutput> {
        let is_new = input.rule_id.is_none();

        // Create the recurring rule with domain validation
        let rule = RecurringRule::new(
            input.days,
            input.start,
            input.end,
            input.availability,
            input.capabilities,
            input.location_constraint,
            input.label,
            input.priority,
        )
        .map_err(|e| crate::application::errors::AppError::ValidationError(e))?;

        // Upsert the rule
        let rule_id = self.schedule_repo.upsert_rule(
            user_id,
            input.template_id,
            input.rule_id,
            rule,
        )?;

        Ok(UpsertRecurringRuleOutput {
            rule_id,
            is_new,
        })
    }
}
