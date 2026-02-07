use crate::domain::Periodicity;
use chrono::{DateTime, Utc, Weekday};

// ========================================================================
// VALIDATION ERRORS
// ========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskValidationError {
    EmptyTitle,
    TitleTooLong { max: usize, actual: usize },
    DescriptionTooLong { max: usize, actual: usize },
    InvalidTimestamps { reason: String },
}

impl std::fmt::Display for TaskValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskValidationError::EmptyTitle => write!(f, "Task title cannot be empty"),
            TaskValidationError::TitleTooLong { max, actual } => {
                write!(f, "Task title too long: {} characters (max: {})", actual, max)
            }
            TaskValidationError::DescriptionTooLong { max, actual } => {
                write!(f, "Task description too long: {} characters (max: {})", actual, max)
            }
            TaskValidationError::InvalidTimestamps { reason } => {
                write!(f, "Invalid timestamps: {}", reason)
            }
        }
    }
}

impl std::error::Error for TaskValidationError {}

// ========================================================================
// TASK STATUS
// ========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is active and should generate occurrences
    Active,
    /// Task is paused (not deleted, but won't generate occurrences)
    Paused,
    /// Task is archived (completed/no longer relevant)
    Archived,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Active
    }
}

// ========================================================================
// TASK PRIORITY
// ========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Urgent = 4,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

// ========================================================================
// TASK AGGREGATE ROOT
// ========================================================================

/// Task represents the definition of something to be done repeatedly
/// 
/// A Task defines WHAT to do and WHEN (via Periodicity).
/// It doesn't track individual completions - that's the job of TaskOccurrence.
/// 
/// Task is the template, TaskOccurrence is the instance.
/// 
/// # Design Decisions
/// - Task is the **aggregate root** in the task management domain
/// - TaskOccurrence entities are managed separately (see task_occurrence.rs)
/// - No `id` field - persistence concerns belong in infrastructure layer
/// - No direct reference to user - multi-tenancy handled in infrastructure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    // ── CORE ATTRIBUTES ─────────────────────────────────────
    title: String,
    description: Option<String>,
    status: TaskStatus,
    priority: TaskPriority,
    
    // ── SCHEDULING ──────────────────────────────────────────
    periodicity: Periodicity,
    
    // ── METADATA ────────────────────────────────────────────
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Task {
    /// Maximum length for task title
    pub const MAX_TITLE_LENGTH: usize = 200;
    
    /// Maximum length for task description
    pub const MAX_DESCRIPTION_LENGTH: usize = 2000;

    /// Creates a new Task with validation
    pub fn new(
        title: String,
        periodicity: Periodicity,
    ) -> Result<Self, TaskValidationError> {
        let now = Utc::now();
        Self::with_timestamps(title, periodicity, now, now)
    }

    /// Creates a Task with specific timestamps (useful for testing/persistence)
    pub fn with_timestamps(
        title: String,
        periodicity: Periodicity,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, TaskValidationError> {
        // Validate title
        if title.trim().is_empty() {
            return Err(TaskValidationError::EmptyTitle);
        }
        if title.len() > Self::MAX_TITLE_LENGTH {
            return Err(TaskValidationError::TitleTooLong {
                max: Self::MAX_TITLE_LENGTH,
                actual: title.len(),
            });
        }

        // Validate timestamps
        if updated_at < created_at {
            return Err(TaskValidationError::InvalidTimestamps {
                reason: "updated_at cannot be before created_at".to_string(),
            });
        }

        Ok(Self {
            title: title.trim().to_string(),
            description: None,
            status: TaskStatus::default(),
            priority: TaskPriority::default(),
            periodicity,
            created_at,
            updated_at,
        })
    }

    // ── GETTERS ─────────────────────────────────────────────

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn status(&self) -> TaskStatus {
        self.status
    }

    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    pub fn periodicity(&self) -> &Periodicity {
        &self.periodicity
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // ── SETTERS (with validation) ──────────────────────────

    pub fn set_title(&mut self, title: String) -> Result<(), TaskValidationError> {
        if title.trim().is_empty() {
            return Err(TaskValidationError::EmptyTitle);
        }
        if title.len() > Self::MAX_TITLE_LENGTH {
            return Err(TaskValidationError::TitleTooLong {
                max: Self::MAX_TITLE_LENGTH,
                actual: title.len(),
            });
        }
        self.title = title.trim().to_string();
        self.touch();
        Ok(())
    }

    pub fn set_description(&mut self, description: Option<String>) -> Result<(), TaskValidationError> {
        if let Some(ref desc) = description {
            if desc.len() > Self::MAX_DESCRIPTION_LENGTH {
                return Err(TaskValidationError::DescriptionTooLong {
                    max: Self::MAX_DESCRIPTION_LENGTH,
                    actual: desc.len(),
                });
            }
        }
        self.description = description.map(|d| d.trim().to_string());
        self.touch();
        Ok(())
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.touch();
    }

    pub fn set_priority(&mut self, priority: TaskPriority) {
        self.priority = priority;
        self.touch();
    }

    pub fn set_periodicity(&mut self, periodicity: Periodicity) {
        self.periodicity = periodicity;
        self.touch();
    }

    // ── DOMAIN BEHAVIORS ────────────────────────────────────

    /// Check if this task should occur on a specific date
    /// (based on periodicity and status)
    /// 
    /// # Parameters
    /// - `date`: The date to check
    /// - `week_start`: First day of the week (from User calendar settings)
    pub fn should_occur_on(&self, date: &DateTime<Utc>, week_start: Weekday) -> bool {
        // Only active tasks generate occurrences
        if self.status != TaskStatus::Active {
            return false;
        }

        // Check if date matches periodicity constraints
        if !self.periodicity.matches_constraints(date, week_start) {
            return false;
        }

        // Check if within timeframe
        self.periodicity.is_within_timeframe(date)
    }

    /// Check if task is currently active
    pub fn is_active(&self) -> bool {
        self.status == TaskStatus::Active
    }

    /// Pause the task (won't generate occurrences)
    pub fn pause(&mut self) {
        self.set_status(TaskStatus::Paused);
    }

    /// Resume a paused task
    pub fn resume(&mut self) {
        if self.status == TaskStatus::Paused {
            self.set_status(TaskStatus::Active);
        }
    }

    /// Archive the task (mark as done/no longer relevant)
    pub fn archive(&mut self) {
        self.set_status(TaskStatus::Archived);
    }

    // ── INTERNAL HELPERS ────────────────────────────────────

    /// Update the updated_at timestamp
    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

// ========================================================================
// TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Periodicity;

    // ── Task Tests ──────────────────────────────────────────

    #[test]
    fn test_task_creation_valid() {
        let periodicity = Periodicity::daily().unwrap();
        let task = Task::new("Buy groceries".to_string(), periodicity);
        assert!(task.is_ok());
    }

    #[test]
    fn test_task_creation_empty_title() {
        let periodicity = Periodicity::daily().unwrap();
        let task = Task::new("   ".to_string(), periodicity);
        assert!(matches!(task, Err(TaskValidationError::EmptyTitle)));
    }

    #[test]
    fn test_task_creation_title_too_long() {
        let periodicity = Periodicity::daily().unwrap();
        let long_title = "a".repeat(Task::MAX_TITLE_LENGTH + 1);
        let task = Task::new(long_title, periodicity);
        assert!(matches!(task, Err(TaskValidationError::TitleTooLong { .. })));
    }

    #[test]
    fn test_task_status_changes() {
        let periodicity = Periodicity::daily().unwrap();
        let mut task = Task::new("Test task".to_string(), periodicity).unwrap();
        
        assert!(task.is_active());
        
        task.pause();
        assert_eq!(task.status(), TaskStatus::Paused);
        assert!(!task.is_active());
        
        task.resume();
        assert_eq!(task.status(), TaskStatus::Active);
        assert!(task.is_active());
        
        task.archive();
        assert_eq!(task.status(), TaskStatus::Archived);
        assert!(!task.is_active());
    }

    #[test]
    fn test_task_should_occur_respects_status() {
        let periodicity = Periodicity::daily().unwrap();
        let mut task = Task::new("Test task".to_string(), periodicity).unwrap();
        
        let date = Utc::now();
        
        // Active task should occur
        assert!(task.should_occur_on(&date, Weekday::Mon));
        
        // Paused task should not occur
        task.pause();
        assert!(!task.should_occur_on(&date, Weekday::Mon));
        
        // Archived task should not occur
        task.set_status(TaskStatus::Archived);
        assert!(!task.should_occur_on(&date, Weekday::Mon));
    }

    #[test]
    fn test_task_priority() {
        let periodicity = Periodicity::daily().unwrap();
        let mut task = Task::new("Test task".to_string(), periodicity).unwrap();
        
        assert_eq!(task.priority(), TaskPriority::Medium);
        
        task.set_priority(TaskPriority::Urgent);
        assert_eq!(task.priority(), TaskPriority::Urgent);
    }
}