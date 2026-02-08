/// Schedule-related DTOs

use crate::domain::entities::schedule::{AvailabilityKind, CapabilitySet, LocationConstraint};
use crate::application::types::{ScheduleTemplateId, RecurringRuleId};
use chrono::{NaiveTime, Weekday};

/// Input for creating a schedule template
#[derive(Debug, Clone)]
pub struct CreateScheduleTemplateInput {
    pub name: String,
    pub description: Option<String>,
}

/// Input for upserting a recurring rule
#[derive(Debug, Clone)]
pub struct UpsertRecurringRuleInput {
    pub template_id: ScheduleTemplateId,
    pub rule_id: Option<RecurringRuleId>, // None for new, Some for update
    pub days: Vec<Weekday>,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub availability: AvailabilityKind,
    pub capabilities: CapabilitySet,
    pub location_constraint: LocationConstraint,
    pub label: Option<String>,
    pub priority: i16,
}

/// Output after creating a schedule template
#[derive(Debug, Clone)]
pub struct CreateScheduleTemplateOutput {
    pub template_id: ScheduleTemplateId,
    pub name: String,
}

/// Output after upserting a recurring rule
#[derive(Debug, Clone)]
pub struct UpsertRecurringRuleOutput {
    pub rule_id: RecurringRuleId,
    pub is_new: bool,
}
