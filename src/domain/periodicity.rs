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
///     reference_date: None,
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
    
    // ── REFERENCE DATE ───────────────────────────────────────
    
    /// Reference date for EveryN* rolling patterns (EveryNDays, EveryNWeeks, etc.)
    /// This is the anchor point from which to count intervals.
    /// 
    /// # Setting the Reference Date
    /// Should be set by the Task layer based on:
    /// 1. First TaskOccurrence date (if any exist)
    /// 2. Otherwise, uses timeframe.start_inclusive if set
    /// 3. Otherwise, uses first date being checked as fallback
    pub reference_date: Option<DateTime<Utc>>,
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
    
    /// Gets the effective reference date for EveryN* constraint calculations
    /// 
    /// # Rules (in priority order):
    /// 1. If reference_date is set (from TaskOccurrence), use it
    /// 2. If timeframe.start_inclusive is set, use it
    /// 3. Use the provided current_date as fallback
    fn get_effective_reference_date(&self, current_date: &DateTime<Utc>) -> DateTime<Utc> {
        // Rule 1: Explicit reference date (set from TaskOccurrence layer)
        if let Some(ref_date) = self.reference_date {
            return ref_date;
        }
        
        // Rule 2: Timeframe start (if set)
        if let Some((start, _)) = self.timeframe {
            return start;
        }
        
        // Rule 3: Fallback to current date being checked
        *current_date
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
            DayConstraint::EveryNDays(n) => {
                let ref_date = self.get_effective_reference_date(date);
                let days_diff = (*date - ref_date).num_days().abs();
                (days_diff % (*n as i64)) == 0
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
            WeekConstraint::EveryNWeeks(n) => {
                let ref_date = self.get_effective_reference_date(date);
                
                // Get the start of the week for both dates (respecting week_start)
                let ref_week_start = Self::get_week_start(&ref_date, self.week_start);
                let date_week_start = Self::get_week_start(date, self.week_start);
                
                // Calculate weeks difference
                let days_diff = (date_week_start - ref_week_start).num_days().abs();
                let weeks_diff = days_diff / 7;
                
                (weeks_diff % (*n as i64)) == 0
            }
            WeekConstraint::SpecificWeeksOfMonthFromFirst(weeks) => {
                let week_of_month = Self::week_of_month_from_first(date, self.week_start);
                // 255 means invalid (belongs to different month)
                if week_of_month == 255 {
                    return false;
                }
                weeks.contains(&week_of_month)
            }
            WeekConstraint::SpecificWeeksOfMonthFromLast(weeks) => {
                let week_of_month = Self::week_of_month_from_last(date, self.week_start);
                // 255 means invalid (belongs to different month)
                if week_of_month == 255 {
                    return false;
                }
                weeks.contains(&week_of_month)
            }
        }
    }
    
    fn matches_month_constraint(&self, date: &DateTime<Utc>, constraint: &MonthConstraint) -> bool {
        match constraint {
            MonthConstraint::EveryMonth => true,
            MonthConstraint::EveryNMonths(n) => {
                let ref_date = self.get_effective_reference_date(date);
                
                // Calculate months difference
                let years_diff = date.year() - ref_date.year();
                let months_diff = (years_diff * 12) + (date.month() as i32 - ref_date.month() as i32);
                
                (months_diff.abs() % (*n as i32)) == 0
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
            YearConstraint::EveryNYears(n) => {
                let ref_date = self.get_effective_reference_date(date);
                let years_diff = (date.year() - ref_date.year()).abs();
                (years_diff % (*n as i32)) == 0
            }
            YearConstraint::SpecificYears(years) => {
                years.contains(&date.year())
            }
        }
    }
    
    // ── HELPER FUNCTIONS ─────────────────────────────────────
        /// Get the start of the week for a given date, based on week_start setting
    /// Returns a DateTime at 00:00:00 on the week_start day
    fn get_week_start(date: &DateTime<Utc>, week_start: Weekday) -> DateTime<Utc> {
        let current_weekday = date.weekday();
        
        // Calculate days to go back to reach week_start
        let days_back = (current_weekday.num_days_from_monday() + 7 
            - week_start.num_days_from_monday()) % 7;
        
        let week_start_date = if days_back == 0 {
            date.date_naive()
        } else {
            date.date_naive() - chrono::Duration::days(days_back as i64)
        };
        
        DateTime::from_naive_utc_and_offset(
            week_start_date.and_hms_opt(0, 0, 0).unwrap(),
            Utc
        )
    }
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
    
    /// Calculate which week of the month (0-indexed) a date falls into,
    /// counting from the first occurrence of week_start.
    /// 
    /// # Week Calculation Rules
    /// - Week 0 starts on the first occurrence of week_start in the month
    /// - Each subsequent week starts 7 days later
    /// - Days before the first week_start belong to the previous month (return 255 as invalid)
    /// - Weeks that overflow into next month still belong to this month
    /// 
    /// # Example
    /// February 2026 with week_start = Monday:
    /// - Feb 1 (Sun): invalid (belongs to January's last week)
    /// - Feb 2-8 (Mon-Sun): Week 0
    /// - Feb 9-15 (Mon-Sun): Week 1
    /// - Feb 16-22 (Mon-Sun): Week 2
    /// - Feb 23-Mar 1 (Mon-Sun): Week 3 (overflow attached to February)
    fn week_of_month_from_first(date: &DateTime<Utc>, week_start: Weekday) -> u8 {
        let year = date.year();
        let month = date.month();
        let day = date.day();
        
        // Get the first day of this month
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday();
        
        // Find the first occurrence of week_start in this month
        let first_week_start_day = if first_weekday == week_start {
            1
        } else {
            // Calculate days forward to reach week_start
            let days_forward = (week_start.num_days_from_monday() + 7 
                - first_weekday.num_days_from_monday()) % 7;
            1 + days_forward as u32
        };
        
        // If date is before first week_start, it belongs to previous month
        if day < first_week_start_day {
            return 255; // Invalid - belongs to previous month
        }
        
        // Calculate which week (0-indexed) since first week_start
        let days_since_first_week_start = day - first_week_start_day;
        (days_since_first_week_start / 7) as u8
    }
    
    /// Calculate which week of the month (0-indexed) a date falls into,
    /// counting backwards from the last complete week ending in the month.
    ///
    /// # Week Calculation Rules
    /// - Week 0 is the last complete week that ends in this month
    /// - Week boundaries respect week_start (week ends on day before week_start)
    /// - Days after the last complete week belong to next month (return 255 as invalid)
    ///
    /// # Example
    /// February 2026 with week_start = Monday (ends on Sunday):
    /// - Feb 1 (Sun): invalid (belongs to previous month's week)
    /// - Feb 2-8 (Mon-Sun): Week 3
    /// - Feb 9-15 (Mon-Sun): Week 2  
    /// - Feb 16-22 (Mon-Sun): Week 1
    /// - Feb 23-28 (Mon-Sat): Week 0 (last week, incomplete in Feb but completes in March)
    fn week_of_month_from_last(date: &DateTime<Utc>, week_start: Weekday) -> u8 {
        let year = date.year();
        let month = date.month();
        let day = date.day();
        
        let naive_date = date.naive_utc().date();
        let last_day = Self::last_day_of_month(naive_date);
        let last_date = NaiveDate::from_ymd_opt(year, month, last_day).unwrap();
        let last_weekday = last_date.weekday();
        
        // Find the last day that is just before week_start (end of week)
        // If week_start is Monday, week ends on Sunday
        let week_end = if week_start == Weekday::Mon {
            Weekday::Sun
        } else {
            // Get previous day
            match week_start {
                Weekday::Tue => Weekday::Mon,
                Weekday::Wed => Weekday::Tue,
                Weekday::Thu => Weekday::Wed,
                Weekday::Fri => Weekday::Thu,
                Weekday::Sat => Weekday::Fri,
                Weekday::Sun => Weekday::Sat,
                Weekday::Mon => Weekday::Sun,
            }
        };
        
        // Find the last occurrence of week_end in this month
        let last_week_end_day = if last_weekday == week_end {
            last_day
        } else {
            // Calculate days backward to reach week_end
            let days_back = (last_weekday.num_days_from_monday() + 7 
                - week_end.num_days_from_monday()) % 7;
            if days_back == 0 {
                // week_end is after last_weekday, so go back a full week
                last_day.saturating_sub(7)
            } else {
                last_day - days_back as u32
            }
        };
        
        // If date is after last complete week, belongs to next month
        if day > last_week_end_day {
            return 255; // Invalid - belongs to next month
        }
        
        // Calculate which week (0-indexed from end) 
        let days_before_last_week_end = last_week_end_day - day;
        (days_before_last_week_end / 7) as u8
    }
    
    /// Get the total number of complete weeks in a month based on week_start
    /// This is useful for validation and understanding month structure
    pub fn weeks_in_month(year: i32, month: u32, week_start: Weekday) -> u8 {
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let first_weekday = first_day.weekday();
        
        // Find first week_start
        let first_week_start_day = if first_weekday == week_start {
            1
        } else {
            let days_forward = (week_start.num_days_from_monday() + 7 
                - first_weekday.num_days_from_monday()) % 7;
            1 + days_forward as u32
        };
        
        // Get last day of month
        let last_day = Self::last_day_of_month(first_day);
        
        // Calculate how many complete weeks fit
        if last_day < first_week_start_day {
            return 0;
        }
        
        let days_from_first_week_start = last_day - first_week_start_day;
        ((days_from_first_week_start / 7) + 1) as u8
    }
}
