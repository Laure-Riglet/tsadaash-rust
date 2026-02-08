/// Application use cases

// User use cases
pub mod register_user;
pub mod update_user_settings;

// Schedule use cases
pub mod create_schedule_template;
pub mod upsert_recurring_rule;
pub mod set_active_schedule_template;

// Task use cases
pub mod create_task;
pub mod update_task;
pub mod complete_occurrence_rep;

// View use cases
pub mod get_day_overview;

// Re-exports
pub use register_user::RegisterUser;
pub use update_user_settings::UpdateUserSettings;
pub use create_schedule_template::CreateScheduleTemplate;
pub use upsert_recurring_rule::UpsertRecurringRule;
pub use set_active_schedule_template::SetActiveScheduleTemplate;
pub use create_task::CreateTask;
pub use update_task::UpdateTask;
pub use complete_occurrence_rep::CompleteOccurrenceRep;
pub use get_day_overview::GetDayOverview;
