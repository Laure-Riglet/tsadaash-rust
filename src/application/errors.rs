/// Application layer errors

use std::fmt;

use crate::application::types::{TaskId, UserId, ScheduleTemplateId, RecurringRuleId};

/// Result type for application operations
pub type AppResult<T> = Result<T, AppError>;

/// Application layer errors
#[derive(Debug)]
pub enum AppError {
    /// User not found
    UserNotFound(UserId),
    
    /// Task not found
    TaskNotFound(TaskId),
    
    /// Schedule template not found
    ScheduleTemplateNotFound(ScheduleTemplateId),
    
    /// Recurring rule not found
    RecurringRuleNotFound(RecurringRuleId),
    
    /// User already exists
    UserAlreadyExists(String),
    
    /// Domain validation error
    ValidationError(String),
    
    /// Authentication failed
    AuthenticationFailed,
    
    /// Generic internal error
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserNotFound(id) => write!(f, "User not found: {}", id),
            Self::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            Self::ScheduleTemplateNotFound(id) => write!(f, "Schedule template not found: {}", id),
            Self::RecurringRuleNotFound(id) => write!(f, "Recurring rule not found: {}", id),
            Self::UserAlreadyExists(username) => write!(f, "User already exists: {}", username),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::AuthenticationFailed => write!(f, "Authentication failed"),
            Self::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

// Convert domain validation errors (String) to AppError
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::ValidationError(msg)
    }
}
