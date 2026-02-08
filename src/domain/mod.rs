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

// Schedule module
pub use entities::schedule::{
    // Core types
    AvailabilityKind,
    AvailabilityLevel,
    CapabilitySet,
    DeviceAccess,
    LocationConstraint,
    Mobility,
    UnavailableReason,
    
    // Template types
    RecurringRule,
    ScheduleTemplate,
    
    // Expansion
    TimeBlock,
    expand_template,
    
    // Matching
    SchedulableTask,
    can_schedule_task_in_block,
    find_candidate_slots,
    
    // Config functions
    busy_flex_max_device,
    busy_flex_max_eyes,
    busy_flex_max_hands,
    busy_flex_max_minutes,
};