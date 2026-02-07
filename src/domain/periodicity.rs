use chrono::{DateTime, Datelike, Month, NaiveDate, Utc, Weekday};

pub mod validation;
pub mod builder;

#[cfg(test)]
mod tests;

// ========================================================================
// CORE REPETITION SETTINGS
// Defines how often a task repeats (frequency) independent of constraints
// ========================================================================

/// Defines the time unit for task repetition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepetitionUnit {
    /// Task repeats multiple times per day
    Day,
    /// Task repeats multiple times per week
    Week,
    /// Task repeats multiple times per month
    Month,
    /// Task repeats multiple times per year
    Year,
    /// No repetition (for unique or custom date tasks)
    None,
}

// ========================================================================
// DAY CONSTRAINTS
// Filter which specific days a task can occur on
// ========================================================================

/// Specifies which week of the month for day constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonthWeekPosition {
    /// Week counting from the start (0-4: first to fifth week)
    FromFirst(u8),
    /// Week counting from the end (0-4: last to fifth-last week)
    FromLast(u8),
}

impl MonthWeekPosition {
    /// Validates that the position is within acceptable bounds (0-4)
    pub fn validate(&self) -> Result<(), validation::ValidationError> {
        let value = match self {
            MonthWeekPosition::FromFirst(v) | MonthWeekPosition::FromLast(v) => *v,
        };
        if value > 4 {
            return Err(validation::ValidationError::InvalidValue {
                field: "MonthWeekPosition".into(),
                value: value.to_string(),
                reason: "Week position must be 0-4".into(),
            });
        }
        Ok(())
    }
}

/// Combines weekday with week-of-month for complex day patterns
/// Example: "First Monday", "Last Friday", "Third Wednesday"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NthWeekdayOfMonth {
    pub weekday: Weekday,
    pub position: MonthWeekPosition,
}

/// Constraints that filter which days a task can occur on
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DayConstraint {
    // ── SIMPLE PATTERNS ──────────────────────────────────────
    
    /// Every day (no filtering)
    EveryDay,
    
    /// Every N days (rolling pattern, e.g., every 3 days)
    /// Value range: 1-366
    EveryNDays(u16),
    
    // ── WEEKDAY PATTERNS ─────────────────────────────────────
    
    /// Specific days of the week (e.g., Monday and Friday)
    /// Must contain 1-7 unique weekdays
    SpecificDaysWeek(Vec<Weekday>),
    
    // ── MONTH DAY PATTERNS ───────────────────────────────────
    
    /// Specific days of month counting from start (0-30)
    /// 0 = 1st day, 1 = 2nd day, etc.
    /// Must contain 1-31 unique values
    SpecificDaysMonthFromFirst(Vec<u8>),
    
    /// Specific days of month counting from end (0-30)
    /// 0 = last day, 1 = second-to-last, etc.
    /// Must contain 1-31 unique values
    SpecificDaysMonthFromLast(Vec<u8>),
    
    /// Specific nth weekdays of month
    /// Example: First Monday, Third Friday, Last Sunday
    /// Must contain 1-20 unique combinations
    SpecificNthWeekdaysMonth(Vec<NthWeekdayOfMonth>),
}

// ========================================================================
// WEEK CONSTRAINTS
// Filter which specific weeks a task can occur in
// ========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeekConstraint {
    /// Every week (no filtering)
    EveryWeek,
    
    /// Every N weeks (rolling pattern)
    /// Value range: 1-52
    EveryNWeeks(u8),
    
    /// Specific weeks of month from start (0-4)
    /// 0 = first week, 1 = second week, etc.
    /// Must contain 1-5 unique values
    SpecificWeeksOfMonthFromFirst(Vec<u8>),
    
    /// Specific weeks of month from end (0-4)
    /// 0 = last week, 1 = second-to-last, etc.
    /// Must contain 1-5 unique values
    SpecificWeeksOfMonthFromLast(Vec<u8>),
}

// ========================================================================
// MONTH CONSTRAINTS
// Filter which specific months a task can occur in
// ========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonthConstraint {
    /// Every month (no filtering)
    EveryMonth,
    
    /// Every N months (rolling pattern)
    /// Value range: 1-12
    EveryNMonths(u8),
    
    /// Specific months (e.g., January and July)
    /// Must contain 1-12 unique months
    SpecificMonths(Vec<Month>),
}

// ========================================================================
// YEAR CONSTRAINTS
// Filter based on year-level patterns
// ========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YearConstraint {
    /// Every year (no filtering)
    EveryYear,
    
    /// Every N years (rolling pattern)
    /// Value range: 1-100
    EveryNYears(u8),
    
    /// Specific years (absolute year numbers)
    /// For rare cases like "only in 2025 and 2030"
    SpecificYears(Vec<i32>),
}

// ========================================================================
// SPECIAL PATTERNS
// For non-periodic or irregular task occurrences
// ========================================================================

/// For tasks with specific dates that don't follow a regular pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomDates {
    /// List of specific dates (must be non-empty and sorted)
    pub dates: Vec<DateTime<Utc>>,
}

impl CustomDates {
    pub fn new(mut dates: Vec<DateTime<Utc>>) -> Result<Self, validation::ValidationError> {
        if dates.is_empty() {
            return Err(validation::ValidationError::InvalidValue {
                field: "CustomDates".into(),
                value: "empty".into(),
                reason: "Must contain at least one date".into(),
            });
        }
        dates.sort();
        dates.dedup();
        Ok(Self { dates })
    }
}

/// For one-time tasks occurring on a single specific date
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueDate {
    pub date: DateTime<Utc>,
}

// ========================================================================
// PERIODICITY CONSTRAINTS
// Composable constraints that work together (AND logic)
// ========================================================================

/// All specified constraints must be satisfied for a date to be valid
/// Example: day_constraint + month_constraint = "Mondays in January"
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PeriodicityConstraints {
    pub day_constraint: Option<DayConstraint>,
    pub week_constraint: Option<WeekConstraint>,
    pub month_constraint: Option<MonthConstraint>,
    pub year_constraint: Option<YearConstraint>,
}

// ========================================================================
// MAIN PERIODICITY STRUCT
// Combines repetition frequency with filtering constraints
// ========================================================================

/// Represents the complete periodicity configuration for a task
/// 
/// # Design Philosophy
/// - `rep_unit` + `rep_per_unit`: HOW OFTEN (frequency)
/// - `constraints`: WHEN IT CAN HAPPEN (filters)
/// - `timeframe`: OVERALL VALIDITY PERIOD
/// 
/// # Examples
/// ```
/// use tsadaash::domain::periodicity::*;
/// use chrono::{Weekday, Month};
/// 
/// let periodicity = Periodicity {
///     rep_unit: RepetitionUnit::Day,
///     rep_per_unit: Some(3),
///     constraints: PeriodicityConstraints {
///         day_constraint: Some(DayConstraint::SpecificDaysWeek(vec![Weekday::Mon])),
///         month_constraint: Some(MonthConstraint::SpecificMonths(vec![Month::January])),
///         ..Default::default()
///     },
///     timeframe: None,
///     week_start: Weekday::Mon,
///     year_start: Month::January,
///     special_pattern: None,
/// };
/// # assert_eq!(periodicity.rep_unit, RepetitionUnit::Day);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Periodicity {
    // ── REPETITION FREQUENCY ─────────────────────────────────
    
    /// Time unit for repetition (Day, Week, Month, Year, or None)
    pub rep_unit: RepetitionUnit,
    
    /// Number of occurrences per unit
    /// - Required when rep_unit != None
    /// - None only valid for RepetitionUnit::None
    /// - Range: 1-255 (u8 max, though practical limits apply per unit)
    pub rep_per_unit: Option<u8>,
    
    // ── CONSTRAINTS (FILTERS) ────────────────────────────────
    
    /// Constraints that filter when the task can occur
    /// All constraints are combined with AND logic
    pub constraints: PeriodicityConstraints,
    
    // ── TIME BOUNDARIES ──────────────────────────────────────
    
    /// Optional validity period for this periodicity
    /// (start_inclusive, end_exclusive)
    pub timeframe: Option<(DateTime<Utc>, DateTime<Utc>)>,
    
    // ── CALENDAR SETTINGS ────────────────────────────────────
    
    /// First day of the week (for week-based calculations)
    /// Default: Monday
    pub week_start: Weekday,
    
    /// First month of the year (for year-based calculations)
    /// Default: January (for fiscal years, could be different)
    pub year_start: Month,
    
    // ── SPECIAL PATTERNS ─────────────────────────────────────
    
    /// For non-periodic patterns (Custom or Unique dates)
    /// When set, rep_unit must be RepetitionUnit::None
    pub special_pattern: Option<SpecialPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialPattern {
    Custom(CustomDates),
    Unique(UniqueDate),
}

// ========================================================================
// IMPLEMENTATION HELPERS
// ========================================================================

impl Periodicity {
    /// Validates the entire periodicity configuration
    /// This is the main entry point for domain validation
    pub fn validate(&self) -> Result<(), validation::ValidationError> {
        validation::validate_periodicity(self)
    }
    
    /// Checks if a specific date matches this periodicity's constraints
    /// Does NOT account for timeframe - call is_within_timeframe separately
    pub fn matches_constraints(&self, date: &DateTime<Utc>) -> bool {
        // Handle special patterns first
        if let Some(pattern) = &self.special_pattern {
            return match pattern {
                SpecialPattern::Custom(custom) => custom.dates.contains(date),
                SpecialPattern::Unique(unique) => unique.date == *date,
            };
        }
        
        // Check each constraint
        if let Some(day) = &self.constraints.day_constraint {
            if !self.matches_day_constraint(date, day) {
                return false;
            }
        }
        
        if let Some(week) = &self.constraints.week_constraint {
            if !self.matches_week_constraint(date, week) {
                return false;
            }
        }
        
        if let Some(month) = &self.constraints.month_constraint {
            if !self.matches_month_constraint(date, month) {
                return false;
            }
        }
        
        if let Some(year) = &self.constraints.year_constraint {
            if !self.matches_year_constraint(date, year) {
                return false;
            }
        }
        
        true
    }
    
    /// Checks if date is within the timeframe (if specified)
    pub fn is_within_timeframe(&self, date: &DateTime<Utc>) -> bool {
        match &self.timeframe {
            Some((start, end)) => date >= start && date < end,
            None => true,
        }
    }
    
    // ── PRIVATE CONSTRAINT MATCHERS ──────────────────────────
    
    fn matches_day_constraint(&self, date: &DateTime<Utc>, constraint: &DayConstraint) -> bool {
        match constraint {
            DayConstraint::EveryDay => true,
            DayConstraint::EveryNDays(_n) => {
                // TODO: Requires reference start date to calculate
                // For now, return true (implement with task tracking)
                true
            }
            DayConstraint::SpecificDaysWeek(weekdays) => {
                weekdays.contains(&date.weekday())
            }
            DayConstraint::SpecificDaysMonthFromFirst(days) => {
                let day_of_month = date.day() - 1; // Convert to 0-indexed
                days.contains(&(day_of_month as u8))
            }
            DayConstraint::SpecificDaysMonthFromLast(days) => {
                let naive_date = date.naive_utc().date();
                let last_day = Self::last_day_of_month(naive_date);
                let days_from_end = last_day - date.day();
                days.contains(&(days_from_end as u8))
            }
            DayConstraint::SpecificNthWeekdaysMonth(patterns) => {
                let weekday = date.weekday();
                
                patterns.iter().any(|pattern| {
                    if pattern.weekday != weekday {
                        return false;
                    }
                    
                    match pattern.position {
                        MonthWeekPosition::FromFirst(n) => {
                            Self::is_nth_weekday_from_first(date, weekday, n)
                        }
                        MonthWeekPosition::FromLast(n) => {
                            Self::is_nth_weekday_from_last(date, weekday, n)
                        }
                    }
                })
            }
        }
    }
    
    fn matches_week_constraint(&self, date: &DateTime<Utc>, constraint: &WeekConstraint) -> bool {
        match constraint {
            WeekConstraint::EveryWeek => true,
            WeekConstraint::EveryNWeeks(_n) => {
                // TODO: Requires reference start date
                true
            }
            WeekConstraint::SpecificWeeksOfMonthFromFirst(weeks) => {
                let week_of_month = Self::week_of_month_from_first(date, self.week_start);
                weeks.contains(&week_of_month)
            }
            WeekConstraint::SpecificWeeksOfMonthFromLast(weeks) => {
                let week_of_month = Self::week_of_month_from_last(date, self.week_start);
                weeks.contains(&week_of_month)
            }
        }
    }
    
    fn matches_month_constraint(&self, date: &DateTime<Utc>, constraint: &MonthConstraint) -> bool {
        match constraint {
            MonthConstraint::EveryMonth => true,
            MonthConstraint::EveryNMonths(_n) => {
                // TODO: Requires reference start date
                true
            }
            MonthConstraint::SpecificMonths(months) => {
                let month = Month::try_from(date.month() as u8).unwrap();
                months.contains(&month)
            }
        }
    }
    
    fn matches_year_constraint(&self, date: &DateTime<Utc>, constraint: &YearConstraint) -> bool {
        match constraint {
            YearConstraint::EveryYear => true,
            YearConstraint::EveryNYears(_n) => {
                // TODO: Requires reference start date
                true
            }
            YearConstraint::SpecificYears(years) => {
                years.contains(&date.year())
            }
        }
    }
    
    // ── HELPER FUNCTIONS ─────────────────────────────────────
    
    fn last_day_of_month(date: NaiveDate) -> u32 {
        NaiveDate::from_ymd_opt(
            date.year(),
            date.month() + 1,
            1,
        )
        .unwrap_or(NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap())
        .pred_opt()
        .unwrap()
        .day()
    }
    
    fn is_nth_weekday_from_first(date: &DateTime<Utc>, _weekday: Weekday, n: u8) -> bool {
        let day = date.day();
        let occurrence = (day - 1) / 7;
        occurrence == n as u32
    }
    
    fn is_nth_weekday_from_last(date: &DateTime<Utc>, _weekday: Weekday, n: u8) -> bool {
        let naive_date = date.naive_utc().date();
        let last_day = Self::last_day_of_month(naive_date);
        let days_from_end = last_day - date.day();
        let occurrence = days_from_end / 7;
        occurrence == n as u32
    }
    
    fn week_of_month_from_first(date: &DateTime<Utc>, _week_start: Weekday) -> u8 {
        let day = date.day();
        ((day - 1) / 7) as u8
    }
    
    fn week_of_month_from_last(date: &DateTime<Utc>, _week_start: Weekday) -> u8 {
        let naive_date = date.naive_utc().date();
        let last_day = Self::last_day_of_month(naive_date);
        let days_from_end = last_day - date.day();
        (days_from_end / 7) as u8
    }
}
