use chrono::{DateTime, Utc, Weekday};
use crate::domain::entities::task::periodicity::Periodicity;
use crate::domain::entities::user::Location;
use crate::domain::entities::schedule::{
    SchedulableTask, AvailabilityLevel, DeviceAccess, Mobility,
};
use crate::config;

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
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    // ── CORE ATTRIBUTES ─────────────────────────────────────
    title: String,
    description: Option<String>,
    status: TaskStatus,
    priority: TaskPriority,
    
    // ── SCHEDULING ──────────────────────────────────────────
    periodicity: Periodicity,
    
    // ── LOCATION REQUIREMENTS ───────────────────────────────
    /// Locations where this task can be performed
    /// Empty = task can be done anywhere (location-free)
    /// Non-empty = task requires being at one of these locations
    locations: Vec<Option<Location>>,
    
    // ── CAPABILITY REQUIREMENTS ─────────────────────────────
    /// Minimum hands availability required
    min_hands: AvailabilityLevel,
    
    /// Minimum eyes availability required
    min_eyes: AvailabilityLevel,
    
    /// Minimum speech availability required
    min_speech: AvailabilityLevel,
    
    /// Minimum cognitive availability required
    min_cognitive: AvailabilityLevel,
    
    /// Minimum device access required
    min_device: DeviceAccess,
    
    /// Allowed mobility states (empty = all allowed)
    allowed_mobility: Vec<Mobility>,
    
    // ── METADATA ────────────────────────────────────────────
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Task {
    /// Maximum length for task title
    pub fn max_title_length() -> usize {
        config::task_max_title_length()
    }
    
    /// Maximum length for task description
    pub fn max_description_length() -> usize {
        config::task_max_description_length()
    }

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
        if title.len() > Self::max_title_length() {
            return Err(TaskValidationError::TitleTooLong {
                max: Self::max_title_length(),
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
            locations: Vec::new(), // Default: location-free
            min_hands: AvailabilityLevel::None, // Default: no hands required
            min_eyes: AvailabilityLevel::None,
            min_speech: AvailabilityLevel::None,
            min_cognitive: AvailabilityLevel::None,
            min_device: DeviceAccess::None, // Default: no device required
            allowed_mobility: Vec::new(), // Default: all mobility states allowed
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

    pub fn locations(&self) -> &[Option<Location>] {
        &self.locations
    }

    pub fn min_hands(&self) -> AvailabilityLevel {
        self.min_hands
    }

    pub fn min_eyes(&self) -> AvailabilityLevel {
        self.min_eyes
    }

    pub fn min_speech(&self) -> AvailabilityLevel {
        self.min_speech
    }

    pub fn min_cognitive(&self) -> AvailabilityLevel {
        self.min_cognitive
    }

    pub fn min_device(&self) -> DeviceAccess {
        self.min_device
    }

    pub fn allowed_mobility(&self) -> &[Mobility] {
        &self.allowed_mobility
    }

    // ── SETTERS (with validation) ──────────────────────────

    pub fn set_title(&mut self, title: String) -> Result<(), TaskValidationError> {
        if title.trim().is_empty() {
            return Err(TaskValidationError::EmptyTitle);
        }
        if title.len() > Self::max_title_length() {
            return Err(TaskValidationError::TitleTooLong {
                max: Self::max_title_length(),
                actual: title.len(),
            });
        }
        self.title = title.trim().to_string();
        self.touch();
        Ok(())
    }

    pub fn set_description(&mut self, description: Option<String>) -> Result<(), TaskValidationError> {
        if let Some(ref desc) = description {
            if desc.len() > Self::max_description_length() {
                return Err(TaskValidationError::DescriptionTooLong {
                    max: Self::max_description_length(),
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

    pub fn set_locations(&mut self, locations: Vec<Option<Location>>) {
        self.locations = locations;
        self.touch();
    }

    pub fn set_min_hands(&mut self, min_hands: AvailabilityLevel) {
        self.min_hands = min_hands;
        self.touch();
    }

    pub fn set_min_eyes(&mut self, min_eyes: AvailabilityLevel) {
        self.min_eyes = min_eyes;
        self.touch();
    }

    pub fn set_min_speech(&mut self, min_speech: AvailabilityLevel) {
        self.min_speech = min_speech;
        self.touch();
    }

    pub fn set_min_cognitive(&mut self, min_cognitive: AvailabilityLevel) {
        self.min_cognitive = min_cognitive;
        self.touch();
    }

    pub fn set_min_device(&mut self, min_device: DeviceAccess) {
        self.min_device = min_device;
        self.touch();
    }

    pub fn set_allowed_mobility(&mut self, allowed_mobility: Vec<Mobility>) {
        self.allowed_mobility = allowed_mobility;
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
// SCHEDULABLE TASK IMPLEMENTATION
// ========================================================================

impl SchedulableTask for Task {
    fn estimated_duration_minutes(&self) -> u32 {
        // Get duration from periodicity's occurrence timing settings
        self.periodicity
            .occurrence_settings
            .as_ref()
            .and_then(|settings| settings.duration)
            .unwrap_or(config::task_default_duration_minutes()) as u32
    }

    fn requires_location(&self) -> bool {
        !self.locations.is_empty()
    }

    fn min_hands(&self) -> AvailabilityLevel {
        self.min_hands
    }

    fn min_eyes(&self) -> AvailabilityLevel {
        self.min_eyes
    }

    fn min_speech(&self) -> AvailabilityLevel {
        self.min_speech
    }

    fn min_cognitive(&self) -> AvailabilityLevel {
        self.min_cognitive
    }

    fn min_device(&self) -> DeviceAccess {
        self.min_device
    }

    fn allowed_mobility(&self) -> Vec<Mobility> {
        self.allowed_mobility.clone()
    }
}

// ========================================================================
// TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::task::Periodicity;

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
        let long_title = "a".repeat(Task::max_title_length() + 1);
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