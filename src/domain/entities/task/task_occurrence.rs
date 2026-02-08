use chrono::{DateTime, Utc};
use super::OccurenceRep;
use crate::config;

// ========================================================================
// VALIDATION ERRORS
// ========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOccurrenceValidationError {
    NotesTooLong { max: usize, actual: usize },
    InvalidTimeWindow { reason: String },
    InvalidRepIndex { expected: u8, actual: u8 },
}

impl std::fmt::Display for TaskOccurrenceValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskOccurrenceValidationError::NotesTooLong { max, actual } => {
                write!(f, "Notes too long: {} characters (max: {})", actual, max)
            }
            TaskOccurrenceValidationError::InvalidTimeWindow { reason } => {
                write!(f, "Invalid time window: {}", reason)
            }
            TaskOccurrenceValidationError::InvalidRepIndex { expected, actual } => {
                write!(f, "Invalid rep index: expected 0-{}, got {}", expected - 1, actual)
            }
        }
    }
}

impl std::error::Error for TaskOccurrenceValidationError {}

// ========================================================================
// OCCURRENCE STATUS
// ========================================================================

/// Overall status of a TaskOccurrence based on its repetitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OccurrenceStatus {
    /// No repetitions have been completed
    NotStarted,
    /// Some (but not all) repetitions have been completed
    InProgress,
    /// All repetitions have been completed
    Completed,
}

// ========================================================================
// TASK OCCURRENCE - A specific instance of a task within a time window
// ========================================================================

/// TaskOccurrence represents a specific instance of a Task within a time window
/// 
/// # Time Windows by Repetition Unit:
/// - **Daily task**: window is one day (00:00:00 to 23:59:59)
/// - **Weekly task**: window is one week (Mon 00:00 to Sun 23:59:59, respecting week_start)
/// - **Monthly task**: window is one month (1st 00:00 to last day 23:59:59)
/// - **Yearly task**: window is one year (Jan 1 00:00 to Dec 31 23:59:59)
/// 
/// # Multiple Repetitions:
/// If Task.periodicity.rep_per_unit is 3, this TaskOccurrence will contain
/// 3 OccurenceReps, each tracking its own completion status.
/// 
/// # Domain Relationships
/// - TaskOccurrence is an entity (identity: task_id + window_start)
/// - Task is the aggregate root
/// - TaskOccurrence cannot exist without a Task
/// - In persistence layer, task_id would link back to Task
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOccurrence {
    // Note: task_id would be added by persistence layer to link back to Task
    
    // ── TIME WINDOW ─────────────────────────────────────────
    
    /// Start of the time window (inclusive)
    window_start: DateTime<Utc>,
    
    /// End of the time window (inclusive)
    /// For daily task: 23:59:59 of the same day
    /// For weekly task: 23:59:59 of the last day of week
    /// For monthly task: 23:59:59 of the last day of month
    /// For yearly task: 23:59:59 of Dec 31
    window_end: DateTime<Utc>,
    
    // ── REPETITIONS ─────────────────────────────────────────
    
    /// All repetitions for this occurrence
    /// Length = Task.periodicity.rep_per_unit
    repetitions: Vec<OccurenceRep>,
    
    // ── OCCURRENCE-LEVEL DATA ───────────────────────────────
    
    /// Optional notes for the entire occurrence
    /// Example: "Good workout session today!" (covers all 3 reps)
    notes: Option<String>,
}

impl TaskOccurrence {
    /// Maximum length for occurrence-level notes
    pub fn max_notes_length() -> usize {
        config::occurrence_max_notes_length()
    }

    /// Creates a new TaskOccurrence for a time window with specified number of repetitions
    /// 
    /// # Arguments
    /// - `window_start`: Start of the time window (inclusive)
    /// - `window_end`: End of the time window (inclusive)
    /// - `rep_count`: Number of repetitions (from Task.periodicity.rep_per_unit)
    pub fn new(
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
        rep_count: u8,
    ) -> Result<Self, TaskOccurrenceValidationError> {
        if window_end < window_start {
            return Err(TaskOccurrenceValidationError::InvalidTimeWindow {
                reason: "window_end must be >= window_start".to_string(),
            });
        }

        let repetitions = (0..rep_count)
            .map(OccurenceRep::new)
            .collect();

        Ok(Self {
            window_start,
            window_end,
            repetitions,
            notes: None,
        })
    }

    // ── GETTERS ─────────────────────────────────────────────

    pub fn window_start(&self) -> DateTime<Utc> {
        self.window_start
    }

    pub fn window_end(&self) -> DateTime<Utc> {
        self.window_end
    }

    pub fn repetitions(&self) -> &[OccurenceRep] {
        &self.repetitions
    }

    pub fn rep_count(&self) -> u8 {
        self.repetitions.len() as u8
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    /// Get the overall status based on all repetitions
    pub fn status(&self) -> OccurrenceStatus {
        let completed_count = self.repetitions.iter()
            .filter(|r| r.is_completed())
            .count();

        match completed_count {
            0 => OccurrenceStatus::NotStarted,
            n if n == self.repetitions.len() => OccurrenceStatus::Completed,
            _ => OccurrenceStatus::InProgress,
        }
    }

    /// Convenience method: is this occurrence fully completed?
    pub fn is_completed(&self) -> bool {
        self.status() == OccurrenceStatus::Completed
    }

    /// Get when the last repetition was completed (if any)
    pub fn last_completed_at(&self) -> Option<DateTime<Utc>> {
        self.repetitions
            .iter()
            .filter_map(|r| r.completed_at())
            .max()
    }

    // ── DOMAIN BEHAVIORS ────────────────────────────────────

    /// Mark a specific repetition as complete
    pub fn mark_rep_complete(&mut self, rep_index: u8) -> Result<(), TaskOccurrenceValidationError> {
        let rep_count = self.rep_count();
        let rep = self.repetitions.get_mut(rep_index as usize)
            .ok_or_else(|| TaskOccurrenceValidationError::InvalidRepIndex {
                expected: rep_count,
                actual: rep_index,
            })?;
        
        rep.mark_complete();
        Ok(())
    }

    /// Mark a specific repetition as incomplete
    pub fn mark_rep_incomplete(&mut self, rep_index: u8) -> Result<(), TaskOccurrenceValidationError> {
        let rep_count = self.rep_count();
        let rep = self.repetitions.get_mut(rep_index as usize)
            .ok_or_else(|| TaskOccurrenceValidationError::InvalidRepIndex {
                expected: rep_count,
                actual: rep_index,
            })?;
        
        rep.mark_incomplete();
        Ok(())
    }

    /// Mark all repetitions as complete
    pub fn mark_all_complete(&mut self) {
        for rep in &mut self.repetitions {
            rep.mark_complete();
        }
    }

    /// Mark all repetitions as incomplete
    pub fn mark_all_incomplete(&mut self) {
        for rep in &mut self.repetitions {
            rep.mark_incomplete();
        }
    }

    /// Set notes for a specific repetition
    pub fn set_rep_notes(
        &mut self,
        rep_index: u8,
        notes: Option<String>,
    ) -> Result<(), TaskOccurrenceValidationError> {
        let rep_count = self.rep_count();
        let rep = self.repetitions.get_mut(rep_index as usize)
            .ok_or_else(|| TaskOccurrenceValidationError::InvalidRepIndex {
                expected: rep_count,
                actual: rep_index,
            })?;
        
        rep.set_notes(notes)
    }

    /// Set notes for the entire occurrence
    pub fn set_notes(&mut self, notes: Option<String>) -> Result<(), TaskOccurrenceValidationError> {
        if let Some(ref n) = notes {
            if n.len() > Self::max_notes_length() {
                return Err(TaskOccurrenceValidationError::NotesTooLong {
                    max: Self::max_notes_length(),
                    actual: n.len(),
                });
            }
        }
        self.notes = notes.map(|n| n.trim().to_string());
        Ok(())
    }

    /// Check if this occurrence is overdue (window has passed and not completed)
    pub fn is_overdue(&self) -> bool {
        !self.is_completed() && Utc::now() > self.window_end
    }

    /// Check if this occurrence is currently active (within time window)
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        now >= self.window_start && now <= self.window_end
    }

    /// Check if this occurrence is in the future
    pub fn is_future(&self) -> bool {
        Utc::now() < self.window_start
    }

    /// Get completion progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.repetitions.is_empty() {
            return 1.0;
        }
        let completed = self.repetitions.iter().filter(|r| r.is_completed()).count();
        completed as f32 / self.repetitions.len() as f32
    }
}

// ========================================================================
// TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_rep_occurrence_creation() {
        let rep = OccurenceRep::new(0);
        assert_eq!(rep.rep_index(), 0);
        assert!(!rep.is_completed());
        assert!(rep.completed_at().is_none());
    }

    #[test]
    fn test_rep_occurrence_completion() {
        let mut rep = OccurenceRep::new(0);
        
        assert!(!rep.is_completed());
        rep.mark_complete();
        assert!(rep.is_completed());
        assert!(rep.completed_at().is_some());
        
        rep.mark_incomplete();
        assert!(!rep.is_completed());
        assert!(rep.completed_at().is_none());
    }

    #[test]
    fn test_occurrence_single_rep() {
        // Daily task with 1 rep per day
        let start = Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 7, 23, 59, 59).unwrap();
        
        let occurrence = TaskOccurrence::new(start, end, 1).unwrap();
        
        assert_eq!(occurrence.rep_count(), 1);
        assert_eq!(occurrence.status(), OccurrenceStatus::NotStarted);
        assert!(!occurrence.is_completed());
    }

    #[test]
    fn test_occurrence_multiple_reps() {
        // Task with 3 reps per window
        let start = Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 7, 23, 59, 59).unwrap();
        
        let mut occurrence = TaskOccurrence::new(start, end, 3).unwrap();
        
        assert_eq!(occurrence.rep_count(), 3);
        assert_eq!(occurrence.status(), OccurrenceStatus::NotStarted);
        assert_eq!(occurrence.progress(), 0.0);
        
        // Complete first rep
        occurrence.mark_rep_complete(0).unwrap();
        assert_eq!(occurrence.status(), OccurrenceStatus::InProgress);
        assert_eq!(occurrence.progress(), 1.0 / 3.0);
        
        // Complete second rep
        occurrence.mark_rep_complete(1).unwrap();
        assert_eq!(occurrence.status(), OccurrenceStatus::InProgress);
        assert_eq!(occurrence.progress(), 2.0 / 3.0);
        
        // Complete third rep
        occurrence.mark_rep_complete(2).unwrap();
        assert_eq!(occurrence.status(), OccurrenceStatus::Completed);
        assert!(occurrence.is_completed());
        assert_eq!(occurrence.progress(), 1.0);
    }

    #[test]
    fn test_occurrence_mark_all() {
        let start = Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 7, 23, 59, 59).unwrap();
        
        let mut occurrence = TaskOccurrence::new(start, end, 3).unwrap();
        
        occurrence.mark_all_complete();
        assert!(occurrence.is_completed());
        assert_eq!(occurrence.progress(), 1.0);
        
        occurrence.mark_all_incomplete();
        assert!(!occurrence.is_completed());
        assert_eq!(occurrence.progress(), 0.0);
    }

    #[test]
    fn test_occurrence_invalid_rep_index() {
        let start = Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 7, 23, 59, 59).unwrap();
        
        let mut occurrence = TaskOccurrence::new(start, end, 2).unwrap();
        
        // Valid indices: 0, 1
        assert!(occurrence.mark_rep_complete(0).is_ok());
        assert!(occurrence.mark_rep_complete(1).is_ok());
        
        // Invalid index: 2
        let result = occurrence.mark_rep_complete(2);
        assert!(matches!(result, Err(TaskOccurrenceValidationError::InvalidRepIndex { .. })));
    }

    #[test]
    fn test_occurrence_notes() {
        let start = Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 7, 23, 59, 59).unwrap();
        
        let mut occurrence = TaskOccurrence::new(start, end, 2).unwrap();
        
        // Occurrence-level notes
        occurrence.set_notes(Some("Great workout day!".to_string())).unwrap();
        assert_eq!(occurrence.notes(), Some("Great workout day!"));
        
        // Rep-level notes
        occurrence.set_rep_notes(0, Some("Did push-ups".to_string())).unwrap();
        occurrence.set_rep_notes(1, Some("Did squats".to_string())).unwrap();
        
        assert_eq!(occurrence.repetitions()[0].notes(), Some("Did push-ups"));
        assert_eq!(occurrence.repetitions()[1].notes(), Some("Did squats"));
    }

    #[test]
    fn test_occurrence_time_window_validation() {
        let start = Utc.with_ymd_and_hms(2026, 2, 7, 23, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap(); // Before start!
        
        let result = TaskOccurrence::new(start, end, 1);
        assert!(matches!(result, Err(TaskOccurrenceValidationError::InvalidTimeWindow { .. })));
    }

    #[test]
    fn test_occurrence_is_active() {
        // Past occurrence
        let past_start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let past_end = Utc.with_ymd_and_hms(2026, 1, 1, 23, 59, 59).unwrap();
        let past = TaskOccurrence::new(past_start, past_end, 1).unwrap();
        assert!(!past.is_active());
        assert!(past.is_overdue()); // Not completed and past
        assert!(!past.is_future());
        
        // Future occurrence
        let future_start = Utc.with_ymd_and_hms(2026, 12, 31, 0, 0, 0).unwrap();
        let future_end = Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59).unwrap();
        let future = TaskOccurrence::new(future_start, future_end, 1).unwrap();
        assert!(!future.is_active());
        assert!(!future.is_overdue());
        assert!(future.is_future());
    }

    #[test]
    fn test_occurrence_last_completed_at() {
        let start = Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 7, 23, 59, 59).unwrap();
        
        let mut occurrence = TaskOccurrence::new(start, end, 3).unwrap();
        
        assert!(occurrence.last_completed_at().is_none());
        
        occurrence.mark_rep_complete(0).unwrap();
        let first_completed = occurrence.last_completed_at();
        assert!(first_completed.is_some());
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        occurrence.mark_rep_complete(2).unwrap();
        let last_completed = occurrence.last_completed_at();
        assert!(last_completed.is_some());
        assert!(last_completed > first_completed);
    }

    #[test]
    fn test_notes_too_long() {
        let start = Utc.with_ymd_and_hms(2026, 2, 7, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 7, 23, 59, 59).unwrap();
        
        let mut occurrence = TaskOccurrence::new(start, end, 1).unwrap();
        
        // Occurrence notes too long
        let long_notes = "a".repeat(TaskOccurrence::max_notes_length() + 1);
        let result = occurrence.set_notes(Some(long_notes));
        assert!(matches!(result, Err(TaskOccurrenceValidationError::NotesTooLong { .. })));
        
        // Rep notes too long
        let long_rep_notes = "b".repeat(OccurenceRep::max_notes_length() + 1);
        let result = occurrence.set_rep_notes(0, Some(long_rep_notes));
        assert!(matches!(result, Err(TaskOccurrenceValidationError::NotesTooLong { .. })));
    }
}
