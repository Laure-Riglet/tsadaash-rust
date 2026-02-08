// ========================================================================
// DOMAIN MODULE
// Pure business logic with no external dependencies
// ========================================================================

pub mod entities;
pub mod tests;

// ========================================================================
// CONVENIENT RE-EXPORTS
// Flatten common types to avoid deep nesting in imports
// ========================================================================

// User aggregate
pub use entities::user::{
    User,
    Timezone,
    TimezoneError,
    Location,
    LocationError,
    GeoCoordinates,
    GeoCoordinatesError,
};

// Task aggregate
pub use entities::task::{
    Task,
    TaskStatus,
    TaskPriority,
    TaskValidationError,
    TaskOccurrence,
    TaskOccurrenceValidationError,
    OccurenceRep,
    
    // Periodicity types
    Periodicity,
    PeriodicityBuilder,
    PeriodicityConstraints,
    PeriodicityValidationError,
    RepetitionUnit,
    DayConstraint,
    WeekConstraint,
    MonthConstraint,
    YearConstraint,
    MonthWeekPosition,
    NthWeekdayOfMonth,
    SpecialPattern,
    CustomDates,
    UniqueDate,
    OccurrenceTimingSettings,
    RepTimingSettings,
};