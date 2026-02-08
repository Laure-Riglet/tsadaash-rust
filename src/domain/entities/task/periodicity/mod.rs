// ========================================================================
// PERIODICITY MODULE
// Complex entity with builder and validation submodules
// ========================================================================

mod types;
pub mod builder;
pub mod validation;

// Re-export all public types from types module
pub use types::{
    // Core enums and structs
    Periodicity,
    PeriodicityConstraints,
    RepetitionUnit,
    SpecialPattern,
    CustomDates,
    UniqueDate,
    
    // Day constraints
    DayConstraint,
    MonthWeekPosition,
    NthWeekdayOfMonth,
    
    // Other constraints
    WeekConstraint,
    MonthConstraint,
    YearConstraint,
    
    // Occurrence timing
    OccurrenceTimingSettings,
    RepTimingSettings,
};

// Re-export builder
pub use builder::PeriodicityBuilder;

// Re-export validation
pub use validation::ValidationError;
