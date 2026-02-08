use chrono::{DateTime, Utc};
use crate::domain::entities::task::TaskOccurrenceValidationError;
use crate::config;

// ========================================================================
// REPETITION OCCURRENCE - A single rep within a TaskOccurrence
// ========================================================================

/// OccurenceRep represents one repetition of a task within a time window
/// 
/// For a task "Exercise 3 times daily", each of the 3 reps is a OccurenceRep.
/// Each rep can be completed independently and have its own notes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccurenceRep {
    /// Index of this repetition (0-based: 0 = first rep, 1 = second rep, etc.)
    rep_index: u8,
    
    /// Whether this specific repetition is completed
    completed: bool,
    
    /// When this repetition was completed
    completed_at: Option<DateTime<Utc>>,
    
    /// Optional notes specific to this repetition
    /// Example: "Did push-ups" vs "Did squats" for different reps
    notes: Option<String>,
}

impl OccurenceRep {
    pub fn max_notes_length() -> usize {
        config::occurrence_rep_max_notes_length()
    }

    /// Creates a new incomplete repetition
    pub fn new(rep_index: u8) -> Self {
        Self {
            rep_index,
            completed: false,
            completed_at: None,
            notes: None,
        }
    }

    // ── GETTERS ─────────────────────────────────────────────

    pub fn rep_index(&self) -> u8 {
        self.rep_index
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

    // ── BEHAVIORS ───────────────────────────────────────────

    pub fn mark_complete(&mut self) {
        if !self.completed {
            self.completed = true;
            self.completed_at = Some(Utc::now());
        }
    }

    pub fn mark_incomplete(&mut self) {
        if self.completed {
            self.completed = false;
            self.completed_at = None;
        }
    }

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
}