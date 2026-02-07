use chrono::{DateTime, Utc};

// ========================================================================
// VALIDATION ERRORS
// ========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOccurrenceValidationError {
    NotesTooLong { max: usize, actual: usize },
}

impl std::fmt::Display for TaskOccurrenceValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskOccurrenceValidationError::NotesTooLong { max, actual } => {
                write!(f, "Notes too long: {} characters (max: {})", actual, max)
            }
        }
    }
}

impl std::error::Error for TaskOccurrenceValidationError {}

// ========================================================================
// TASK OCCURRENCE - A specific instance of a task
// ========================================================================

/// TaskOccurrence represents a specific instance of a Task on a specific date
/// 
/// While Task defines "Exercise daily", TaskOccurrence represents
/// "Exercise on 2026-02-06" with its own completion status, notes, etc.
/// 
/// # Domain Relationships
/// - TaskOccurrence is an entity (has identity: task_id + scheduled_date)
/// - Task is the aggregate root
/// - TaskOccurrence cannot exist without a Task
/// - In persistence layer, task_id would link back to Task
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOccurrence {
    // Note: task_id would be added by persistence layer to link back to Task
    
    // ── OCCURRENCE DATE ─────────────────────────────────────
    scheduled_date: DateTime<Utc>,
    
    // ── COMPLETION STATUS ───────────────────────────────────
    completed: bool,
    completed_at: Option<DateTime<Utc>>,
    
    // ── OCCURRENCE-SPECIFIC DATA ────────────────────────────
    notes: Option<String>,
    
    // ── METADATA ────────────────────────────────────────────
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TaskOccurrence {
    /// Maximum length for occurrence notes
    pub const MAX_NOTES_LENGTH: usize = 1000;

    /// Creates a new TaskOccurrence for a specific date
    pub fn new(scheduled_date: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            scheduled_date,
            completed: false,
            completed_at: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a TaskOccurrence with specific timestamps (for persistence)
    pub fn with_timestamps(
        scheduled_date: DateTime<Utc>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            scheduled_date,
            completed: false,
            completed_at: None,
            notes: None,
            created_at,
            updated_at,
        }
    }

    // ── GETTERS ─────────────────────────────────────────────

    pub fn scheduled_date(&self) -> DateTime<Utc> {
        self.scheduled_date
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // ── DOMAIN BEHAVIORS ────────────────────────────────────

    /// Mark this occurrence as completed
    pub fn mark_complete(&mut self) {
        if !self.completed {
            self.completed = true;
            self.completed_at = Some(Utc::now());
            self.touch();
        }
    }

    /// Mark this occurrence as incomplete (undo)
    pub fn mark_incomplete(&mut self) {
        if self.completed {
            self.completed = false;
            self.completed_at = None;
            self.touch();
        }
    }

    /// Toggle completion status
    pub fn toggle_completion(&mut self) {
        if self.completed {
            self.mark_incomplete();
        } else {
            self.mark_complete();
        }
    }

    /// Set notes for this occurrence
    pub fn set_notes(&mut self, notes: Option<String>) -> Result<(), TaskOccurrenceValidationError> {
        if let Some(ref n) = notes {
            if n.len() > Self::MAX_NOTES_LENGTH {
                return Err(TaskOccurrenceValidationError::NotesTooLong {
                    max: Self::MAX_NOTES_LENGTH,
                    actual: n.len(),
                });
            }
        }
        self.notes = notes.map(|n| n.trim().to_string());
        self.touch();
        Ok(())
    }

    /// Check if this occurrence is overdue
    pub fn is_overdue(&self) -> bool {
        !self.completed && Utc::now() > self.scheduled_date
    }

    /// Check if this occurrence is due today
    pub fn is_due_today(&self) -> bool {
        let now = Utc::now();
        self.scheduled_date.date_naive() == now.date_naive()
    }

    // ── INTERNAL HELPERS ────────────────────────────────────

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

    #[test]
    fn test_occurrence_creation() {
        let date = Utc::now();
        let occurrence = TaskOccurrence::new(date);
        
        assert_eq!(occurrence.scheduled_date(), date);
        assert!(!occurrence.is_completed());
        assert!(occurrence.completed_at().is_none());
    }

    #[test]
    fn test_occurrence_completion() {
        let date = Utc::now();
        let mut occurrence = TaskOccurrence::new(date);
        
        assert!(!occurrence.is_completed());
        
        occurrence.mark_complete();
        assert!(occurrence.is_completed());
        assert!(occurrence.completed_at().is_some());
        
        occurrence.mark_incomplete();
        assert!(!occurrence.is_completed());
        assert!(occurrence.completed_at().is_none());
    }

    #[test]
    fn test_occurrence_toggle() {
        let date = Utc::now();
        let mut occurrence = TaskOccurrence::new(date);
        
        assert!(!occurrence.is_completed());
        
        occurrence.toggle_completion();
        assert!(occurrence.is_completed());
        
        occurrence.toggle_completion();
        assert!(!occurrence.is_completed());
    }

    #[test]
    fn test_occurrence_notes() {
        let date = Utc::now();
        let mut occurrence = TaskOccurrence::new(date);
        
        assert!(occurrence.notes().is_none());
        
        let result = occurrence.set_notes(Some("Did extra reps today!".to_string()));
        assert!(result.is_ok());
        assert_eq!(occurrence.notes(), Some("Did extra reps today!"));
        
        occurrence.set_notes(None).unwrap();
        assert!(occurrence.notes().is_none());
    }

    #[test]
    fn test_occurrence_notes_too_long() {
        let date = Utc::now();
        let mut occurrence = TaskOccurrence::new(date);
        
        let long_notes = "a".repeat(TaskOccurrence::MAX_NOTES_LENGTH + 1);
        let result = occurrence.set_notes(Some(long_notes));
        
        assert!(matches!(result, Err(TaskOccurrenceValidationError::NotesTooLong { .. })));
    }
}
