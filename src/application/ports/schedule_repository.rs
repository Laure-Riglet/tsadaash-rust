/// Schedule repository port

use crate::application::errors::AppResult;
use crate::application::types::{ScheduleTemplateId, RecurringRuleId, UserId};
use crate::domain::entities::schedule::{ScheduleTemplate, RecurringRule};

/// Trait for schedule template persistence operations
pub trait ScheduleRepository {
    /// Save a new schedule template for a user
    fn save_template(&mut self, user_id: UserId, template: ScheduleTemplate) -> AppResult<ScheduleTemplateId>;
    
    /// Find a schedule template by ID
    fn find_template(&self, user_id: UserId, template_id: ScheduleTemplateId) -> AppResult<ScheduleTemplate>;
    
    /// Update a schedule template
    fn update_template(&mut self, user_id: UserId, template_id: ScheduleTemplateId, template: ScheduleTemplate) -> AppResult<()>;
    
    /// Delete a schedule template
    fn delete_template(&mut self, user_id: UserId, template_id: ScheduleTemplateId) -> AppResult<()>;
    
    /// List all schedule templates for a user
    fn list_templates_by_user(&self, user_id: UserId) -> AppResult<Vec<(ScheduleTemplateId, ScheduleTemplate)>>;
    
    /// Upsert a recurring rule in a template
    /// Returns the rule ID (new or existing)
    fn upsert_rule(&mut self, user_id: UserId, template_id: ScheduleTemplateId, rule_id: Option<RecurringRuleId>, rule: RecurringRule) -> AppResult<RecurringRuleId>;
    
    /// Remove a recurring rule from a template
    fn remove_rule(&mut self, user_id: UserId, template_id: ScheduleTemplateId, rule_id: RecurringRuleId) -> AppResult<()>;
}
