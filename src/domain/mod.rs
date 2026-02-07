pub mod timezone;
pub use timezone::{Timezone, TimezoneError};

pub mod location;
pub use location::{Location, LocationError, GeoCoordinates, GeoCoordinatesError};

pub mod periodicity;
pub use periodicity::{
    Periodicity,
    PeriodicityConstraints,
    RepetitionUnit,
    DayConstraint,
    WeekConstraint,
    MonthConstraint,
    YearConstraint,
    NthWeekdayOfMonth,
    MonthWeekPosition,
    SpecialPattern,
    CustomDates,
    UniqueDate,
};
// Re-export builder and validation for convenience
pub use periodicity::builder::PeriodicityBuilder;
pub use periodicity::validation::ValidationError;

pub mod task;
pub use task::{
    Task,
    TaskStatus,
    TaskPriority,
    TaskValidationError,
};

pub mod task_occurrence;
pub use task_occurrence::{
    TaskOccurrence,
    TaskOccurrenceValidationError,
};

pub mod user;
pub use user::User;
