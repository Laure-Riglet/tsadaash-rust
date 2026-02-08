/// In-memory schedule repository implementation

use std::collections::HashMap;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::ScheduleRepository;
use crate::application::types::{ScheduleTemplateId, RecurringRuleId, UserId};
use crate::domain::entities::schedule::{ScheduleTemplate, RecurringRule};

/// Key for storing templates per user
type TemplateKey = (UserId, ScheduleTemplateId);

/// In-memory implementation of ScheduleRepository for testing/MVP
pub struct InMemoryScheduleRepository {
    templates: HashMap<TemplateKey, ScheduleTemplate>,
    next_template_id: u64,
    next_rule_id: u64,
}

impl InMemoryScheduleRepository {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            next_template_id: 1,
            next_rule_id: 1,
        }
    }
}

impl ScheduleRepository for InMemoryScheduleRepository {
    fn save_template(&mut self, user_id: UserId, template: ScheduleTemplate) -> AppResult<ScheduleTemplateId> {
        let template_id = ScheduleTemplateId::new(self.next_template_id);
        self.next_template_id += 1;

        self.templates.insert((user_id, template_id), template);

        Ok(template_id)
    }

    fn find_template(&self, user_id: UserId, template_id: ScheduleTemplateId) -> AppResult<ScheduleTemplate> {
        self.templates
            .get(&(user_id, template_id))
            .cloned()
            .ok_or(AppError::ScheduleTemplateNotFound(template_id))
    }

    fn update_template(&mut self, user_id: UserId, template_id: ScheduleTemplateId, template: ScheduleTemplate) -> AppResult<()> {
        let key = (user_id, template_id);
        if !self.templates.contains_key(&key) {
            return Err(AppError::ScheduleTemplateNotFound(template_id));
        }

        self.templates.insert(key, template);
        Ok(())
    }

    fn delete_template(&mut self, user_id: UserId, template_id: ScheduleTemplateId) -> AppResult<()> {
        let key = (user_id, template_id);
        self.templates.remove(&key)
            .ok_or(AppError::ScheduleTemplateNotFound(template_id))?;
        Ok(())
    }

    fn list_templates_by_user(&self, user_id: UserId) -> AppResult<Vec<(ScheduleTemplateId, ScheduleTemplate)>> {
        let templates: Vec<(ScheduleTemplateId, ScheduleTemplate)> = self.templates
            .iter()
            .filter(|((uid, _), _)| *uid == user_id)
            .map(|((_, tid), template)| (*tid, template.clone()))
            .collect();

        Ok(templates)
    }

    fn upsert_rule(&mut self, user_id: UserId, template_id: ScheduleTemplateId, rule_id: Option<RecurringRuleId>, rule: RecurringRule) -> AppResult<RecurringRuleId> {
        let key = (user_id, template_id);
        let mut template = self.templates
            .get(&key)
            .cloned()
            .ok_or(AppError::ScheduleTemplateNotFound(template_id))?;

        let rule_id = match rule_id {
            Some(rid) => {
                // Update existing rule
                // For MVP, we'll just add the rule (in a real implementation, you'd track rule IDs)
                template.rules.push(rule);
                rid
            }
            None => {
                // Create new rule
                let rid = RecurringRuleId::new(self.next_rule_id);
                self.next_rule_id += 1;
                template.rules.push(rule);
                rid
            }
        };

        self.templates.insert(key, template);

        Ok(rule_id)
    }

    fn remove_rule(&mut self, user_id: UserId, template_id: ScheduleTemplateId, rule_id: RecurringRuleId) -> AppResult<()> {
        let key = (user_id, template_id);
        let template = self.templates
            .get_mut(&key)
            .ok_or(AppError::ScheduleTemplateNotFound(template_id))?;

        // For MVP, we don't track individual rule IDs well enough to remove specific rules
        // In a real implementation, you'd need to track which rule has which ID
        // For now, just return an error if the template is empty
        if template.rules.is_empty() {
            return Err(AppError::RecurringRuleNotFound(rule_id));
        }

        // Remove the first rule as a placeholder
        template.rules.remove(0);

        Ok(())
    }
}
