/// Application layer

pub mod dto;
pub mod errors;
pub mod ports;
pub mod types;
pub mod use_cases;

// Re-export commonly used items
pub use errors::{AppError, AppResult};
pub use types::{UserId, TaskId, ScheduleTemplateId, RecurringRuleId};
