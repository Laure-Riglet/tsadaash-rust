use chrono::{DateTime, Utc, Weekday, Month, TimeZone};
use crate::domain::validators::periodicity_validator;
use crate::domain::entities::task::periodicity::{OccurrenceTimingSettings, NthWeekdayOfMonth, RepetitionUnit};
use crate::domain::entities::task::{
    DayConstraint, MonthConstraint, MonthWeekPosition, Periodicity, PeriodicityConstraints,
    SpecialPattern, WeekConstraint, YearConstraint, CustomDates, UniqueDate,
};

// ========================================================================
// PERIODICITY BUILDER
// Safe, fluent API for constructing Periodicity instances
// ========================================================================

/// Builder for creating validated Periodicity instances
/// 
/// # Example
/// ```
/// use tsadaash::domain::builders::periodicity_builder::PeriodicityBuilder;
/// use chrono::{Weekday, Month};
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let periodicity = PeriodicityBuilder::new()
///     .daily(3)  // 3 times per day
///     .on_weekdays(vec![Weekday::Mon, Weekday::Wed, Weekday::Fri])
///     .in_months(vec![Month::January, Month::February])
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PeriodicityBuilder {
    rep_unit: Option<RepetitionUnit>,
    rep_per_unit: Option<u8>,
    occurrence_settings: Option<OccurrenceTimingSettings>,
    day_constraint: Option<DayConstraint>,
    week_constraint: Option<WeekConstraint>,
    month_constraint: Option<MonthConstraint>,
    year_constraint: Option<YearConstraint>,
    timeframe: Option<(DateTime<Utc>, DateTime<Utc>)>,
    special_pattern: Option<SpecialPattern>,
    reference_date: Option<DateTime<Utc>>,
}

impl Default for PeriodicityBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PeriodicityBuilder {
    /// Creates a new builder with default settings
    pub fn new() -> Self {
        Self {
            rep_unit: None,
            rep_per_unit: None,
            occurrence_settings: None,
            day_constraint: None,
            week_constraint: None,
            month_constraint: None,
            year_constraint: None,
            timeframe: None,
            special_pattern: None,
            reference_date: None,
        }
    }
    
    // ────────────────────────────────────────────────────────
    // REPETITION UNIT SETTERS
    // ────────────────────────────────────────────────────────
    
    /// Sets daily repetition (N times per day)
    pub fn daily(mut self, count: u8) -> Self {
        self.rep_unit = Some(RepetitionUnit::Day);
        self.rep_per_unit = Some(count);
        self
    }
    
    /// Sets weekly repetition (N times per week)
    pub fn weekly(mut self, count: u8) -> Self {
        self.rep_unit = Some(RepetitionUnit::Week);
        self.rep_per_unit = Some(count);
        self
    }
    
    /// Sets monthly repetition (N times per month)
    pub fn monthly(mut self, count: u8) -> Self {
        self.rep_unit = Some(RepetitionUnit::Month);
        self.rep_per_unit = Some(count);
        self
    }
    
    /// Sets yearly repetition (N times per year)
    pub fn yearly(mut self, count: u8) -> Self {
        self.rep_unit = Some(RepetitionUnit::Year);
        self.rep_per_unit = Some(count);
        self
    }
    
    // ────────────────────────────────────────────────────────
    // DAY CONSTRAINT SETTERS
    // ────────────────────────────────────────────────────────
    
    /// No day filtering (every day is valid)
    pub fn every_day(mut self) -> Self {
        self.day_constraint = Some(DayConstraint::EveryDay);
        self
    }
    
    /// Occurs every N days (rolling pattern)
    pub fn every_n_days(mut self, n: u16) -> Self {
        self.day_constraint = Some(DayConstraint::EveryNDays(n));
        self
    }
    
    /// Occurs on specific weekdays
    pub fn on_weekdays(mut self, weekdays: Vec<Weekday>) -> Self {
        self.day_constraint = Some(DayConstraint::SpecificDaysWeek(weekdays));
        self
    }
    
    /// Occurs on specific days of the month (1-31)
    pub fn on_month_days(mut self, days: Vec<u8>) -> Self {
        // Convert 1-indexed to 0-indexed
        let zero_indexed: Vec<u8> = days.into_iter().map(|d| d.saturating_sub(1)).collect();
        self.day_constraint = Some(DayConstraint::SpecificDaysMonthFromFirst(zero_indexed));
        self
    }
    
    /// Occurs on specific days from end of month (1 = last day, 2 = second-to-last, etc.)
    pub fn on_month_days_from_end(mut self, days: Vec<u8>) -> Self {
        // Convert 1-indexed to 0-indexed
        let zero_indexed: Vec<u8> = days.into_iter().map(|d| d.saturating_sub(1)).collect();
        self.day_constraint = Some(DayConstraint::SpecificDaysMonthFromLast(zero_indexed));
        self
    }
    
    /// Occurs on specific nth weekdays of the month
    /// Example: first_monday(), last_friday()
    pub fn on_nth_weekdays(mut self, patterns: Vec<NthWeekdayOfMonth>) -> Self {
        self.day_constraint = Some(DayConstraint::SpecificNthWeekdaysMonth(patterns));
        self
    }
    
    // ────────────────────────────────────────────────────────
    // WEEK CONSTRAINT SETTERS
    // ────────────────────────────────────────────────────────
    
    /// No week filtering (every week is valid)
    pub fn every_week(mut self) -> Self {
        self.week_constraint = Some(WeekConstraint::EveryWeek);
        self
    }
    
    /// Occurs every N weeks
    pub fn every_n_weeks(mut self, n: u8) -> Self {
        self.week_constraint = Some(WeekConstraint::EveryNWeeks(n));
        self
    }
    
    /// Occurs on specific weeks of the month from start (1-5)
    pub fn on_weeks_of_month(mut self, weeks: Vec<u8>) -> Self {
        // Convert 1-indexed to 0-indexed
        let zero_indexed: Vec<u8> = weeks.into_iter().map(|w| w.saturating_sub(1)).collect();
        self.week_constraint = Some(WeekConstraint::SpecificWeeksOfMonthFromFirst(zero_indexed));
        self
    }
    
    /// Occurs on specific weeks from end of month (1 = last week, 2 = second-to-last, etc.)
    pub fn on_weeks_of_month_from_end(mut self, weeks: Vec<u8>) -> Self {
        // Convert 1-indexed to 0-indexed
        let zero_indexed: Vec<u8> = weeks.into_iter().map(|w| w.saturating_sub(1)).collect();
        self.week_constraint = Some(WeekConstraint::SpecificWeeksOfMonthFromLast(zero_indexed));
        self
    }
    
    // ────────────────────────────────────────────────────────
    // MONTH CONSTRAINT SETTERS
    // ────────────────────────────────────────────────────────
    
    /// No month filtering (every month is valid)
    pub fn every_month(mut self) -> Self {
        self.month_constraint = Some(MonthConstraint::EveryMonth);
        self
    }
    
    /// Occurs every N months
    pub fn every_n_months(mut self, n: u8) -> Self {
        self.month_constraint = Some(MonthConstraint::EveryNMonths(n));
        self
    }
    
    /// Occurs in specific months
    pub fn in_months(mut self, months: Vec<Month>) -> Self {
        self.month_constraint = Some(MonthConstraint::SpecificMonths(months));
        self
    }
    
    // ────────────────────────────────────────────────────────
    // YEAR CONSTRAINT SETTERS
    // ────────────────────────────────────────────────────────
    
    /// No year filtering (every year is valid)
    pub fn every_year(mut self) -> Self {
        self.year_constraint = Some(YearConstraint::EveryYear);
        self
    }
    
    /// Occurs every N years
    pub fn every_n_years(mut self, n: u8) -> Self {
        self.year_constraint = Some(YearConstraint::EveryNYears(n));
        self
    }
    
    /// Occurs in specific years
    pub fn in_years(mut self, years: Vec<i32>) -> Self {
        self.year_constraint = Some(YearConstraint::SpecificYears(years));
        self
    }
    
    // ────────────────────────────────────────────────────────
    // SPECIAL PATTERN SETTERS
    // ────────────────────────────────────────────────────────
    
    /// One-time task on a specific date
    pub fn unique(mut self, date: DateTime<Utc>) -> Self {
        self.rep_unit = Some(RepetitionUnit::None);
        self.special_pattern = Some(SpecialPattern::Unique(UniqueDate { date }));
        self
    }
    
    /// Custom dates without regular pattern
    pub fn custom_dates(mut self, dates: Vec<DateTime<Utc>>) -> Result<Self, periodicity_validator::ValidationError> {
        let custom = CustomDates::new(dates)?;
        self.rep_unit = Some(RepetitionUnit::None);
        self.special_pattern = Some(SpecialPattern::Custom(custom));
        Ok(self)
    }
    
    // ────────────────────────────────────────────────────────
    // TIMEFRAME SETTERS
    // ────────────────────────────────────────────────────────
    
    /// Sets the validity period for this periodicity
    pub fn between(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.timeframe = Some((start, end));
        self
    }
    
    /// Sets start date with no end
    pub fn starting_from(mut self, start: DateTime<Utc>) -> Self {
        let far_future = Utc.with_ymd_and_hms(2200, 12, 31, 23, 59, 59).unwrap();
        self.timeframe = Some((start, far_future));
        self
    }
    
    /// Sets end date with no explicit start
    pub fn until(mut self, end: DateTime<Utc>) -> Self {
        let far_past = Utc.with_ymd_and_hms(1900, 1, 1, 0, 0, 0).unwrap();
        self.timeframe = Some((far_past, end));
        self
    }
    
    // ────────────────────────────────────────────────────────
    // REFERENCE DATE
    // ────────────────────────────────────────────────────────
    
    /// Sets the reference date for EveryN* rolling patterns
    /// This is the anchor point from which intervals are counted.
    /// 
    /// # Usage
    /// Typically set by the Task layer based on:
    /// 1. First TaskOccurrence date (if any exist)
    /// 2. Otherwise relies on timeframe.start_inclusive
    /// 3. Otherwise uses date being checked as fallback
    pub fn with_reference_date(mut self, date: DateTime<Utc>) -> Self {
        self.reference_date = Some(date);
        self
    }
    
    // ────────────────────────────────────────────────────────
    // OCCURRENCE TIMING SETTINGS
    // ────────────────────────────────────────────────────────
    
    /// Sets occurrence timing settings (duration, time windows, per-rep settings)
    /// 
    /// # Example
    /// ```
    /// use tsadaash::domain::builders::periodicity_builder::PeriodicityBuilder;
    /// use tsadaash::domain::entities::task::periodicity::OccurrenceTimingSettings;
    /// use chrono::NaiveTime;
    /// 
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let settings = OccurrenceTimingSettings {
    ///     duration: Some(30), // 30 minutes
    ///     not_before: Some(NaiveTime::from_hms_opt(6, 0, 0).unwrap()),
    ///     best_before: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
    ///     rep_timing_settings: None,
    /// };
    /// 
    /// let periodicity = PeriodicityBuilder::new()
    ///     .daily(1)
    ///     .with_occurrence_settings(settings)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_occurrence_settings(mut self, settings: OccurrenceTimingSettings) -> Self {
        self.occurrence_settings = Some(settings);
        self
    }
    
    // ────────────────────────────────────────────────────────
    // BUILD
    // ────────────────────────────────────────────────────────
    
    /// Builds and validates the Periodicity instance
    pub fn build(self) -> Result<Periodicity, periodicity_validator::ValidationError> {
        let periodicity = Periodicity {
            rep_unit: self.rep_unit.unwrap_or(RepetitionUnit::None),
            rep_per_unit: self.rep_per_unit,
            occurrence_settings: self.occurrence_settings,
            constraints: PeriodicityConstraints {
                day_constraint: self.day_constraint,
                week_constraint: self.week_constraint,
                month_constraint: self.month_constraint,
                year_constraint: self.year_constraint,
            },
            timeframe: self.timeframe,
            special_pattern: self.special_pattern,
            reference_date: self.reference_date,
        };
        
        // Validate before returning
        periodicity.validate()?;
        
        Ok(periodicity)
    }
}

// ========================================================================
// CONVENIENCE CONSTRUCTORS
// Quick shortcuts for common patterns
// ========================================================================

impl Periodicity {
    /// Creates a daily task (once per day, every day)
    pub fn daily() -> Result<Self, periodicity_validator::ValidationError> {
        PeriodicityBuilder::new()
            .daily(1)
            .every_day()
            .build()
    }
    
    /// Creates a weekly task (once per week, every week)
    pub fn weekly() -> Result<Self, periodicity_validator::ValidationError> {
        PeriodicityBuilder::new()
            .weekly(1)
            .every_week()
            .build()
    }
    
    /// Creates a monthly task (once per month, every month)
    pub fn monthly() -> Result<Self, periodicity_validator::ValidationError> {
        PeriodicityBuilder::new()
            .monthly(1)
            .every_month()
            .build()
    }
    
    /// Creates a yearly task (once per year, every year)
    pub fn yearly() -> Result<Self, periodicity_validator::ValidationError> {
        PeriodicityBuilder::new()
            .yearly(1)
            .every_year()
            .build()
    }
    
    /// Creates a one-time task
    pub fn unique(date: DateTime<Utc>) -> Result<Self, periodicity_validator::ValidationError> {
        PeriodicityBuilder::new()
            .unique(date)
            .build()
    }
    
    /// Creates a task on specific weekdays (e.g., Monday, Wednesday, Friday)
    pub fn on_weekdays(weekdays: Vec<Weekday>) -> Result<Self, periodicity_validator::ValidationError> {
        PeriodicityBuilder::new()
            .daily(1)
            .on_weekdays(weekdays)
            .build()
    }
    
    /// Creates a task on specific days of the month (1-indexed: 1 = 1st, 15 = 15th, etc.)
    pub fn on_days_of_month(days: Vec<u8>) -> Result<Self, periodicity_validator::ValidationError> {
        PeriodicityBuilder::new()
            .daily(1)
            .on_month_days(days)
            .build()
    }
}

// ========================================================================
// HELPER CONSTRUCTORS FOR NthWeekdayOfMonth
// ========================================================================

impl NthWeekdayOfMonth {
    /// First occurrence of weekday in month (e.g., first Monday)
    pub fn first(weekday: Weekday) -> Self {
        Self {
            weekday,
            position: MonthWeekPosition::FromFirst(0),
        }
    }
    
    /// Second occurrence of weekday in month
    pub fn second(weekday: Weekday) -> Self {
        Self {
            weekday,
            position: MonthWeekPosition::FromFirst(1),
        }
    }
    
    /// Third occurrence of weekday in month
    pub fn third(weekday: Weekday) -> Self {
        Self {
            weekday,
            position: MonthWeekPosition::FromFirst(2),
        }
    }
    
    /// Fourth occurrence of weekday in month
    pub fn fourth(weekday: Weekday) -> Self {
        Self {
            weekday,
            position: MonthWeekPosition::FromFirst(3),
        }
    }
    
    /// Last occurrence of weekday in month
    pub fn last(weekday: Weekday) -> Self {
        Self {
            weekday,
            position: MonthWeekPosition::FromLast(0),
        }
    }
    
    /// Second-to-last occurrence of weekday in month
    pub fn second_last(weekday: Weekday) -> Self {
        Self {
            weekday,
            position: MonthWeekPosition::FromLast(1),
        }
    }
}

// ========================================================================
// UNIT TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, Weekday, Month};
    
    #[test]
    fn test_builder_daily_on_weekdays() {
        let periodicity = PeriodicityBuilder::new()
            .daily(1)
            .on_weekdays(vec![Weekday::Mon, Weekday::Wed, Weekday::Fri])
            .build()
            .unwrap();
        
        assert_eq!(periodicity.rep_unit, RepetitionUnit::Day);
        assert_eq!(periodicity.rep_per_unit, Some(1));
    }
    
    #[test]
    fn test_builder_monthly_specific_days() {
        let periodicity = PeriodicityBuilder::new()
            .daily(1)
            .on_month_days(vec![13, 24])
            .in_months(vec![Month::January, Month::February])
            .build()
            .unwrap();
        
        assert_eq!(periodicity.rep_unit, RepetitionUnit::Day);
        assert!(periodicity.constraints.month_constraint.is_some());
    }
    
    #[test]
    fn test_convenience_daily() {
        let periodicity = Periodicity::daily().unwrap();
        assert_eq!(periodicity.rep_unit, RepetitionUnit::Day);
        assert_eq!(periodicity.rep_per_unit, Some(1));
    }
    
    #[test]
    fn test_nth_weekday_constructors() {
        let first_monday = NthWeekdayOfMonth::first(Weekday::Mon);
        assert_eq!(first_monday.weekday, Weekday::Mon);
        assert_eq!(first_monday.position, MonthWeekPosition::FromFirst(0));
        
        let last_friday = NthWeekdayOfMonth::last(Weekday::Fri);
        assert_eq!(last_friday.weekday, Weekday::Fri);
        assert_eq!(last_friday.position, MonthWeekPosition::FromLast(0));
    }
    
    #[test]
    fn test_unique_date() {
        let date = Utc::now();
        let periodicity = Periodicity::unique(date).unwrap();
        
        assert_eq!(periodicity.rep_unit, RepetitionUnit::None);
        assert!(periodicity.special_pattern.is_some());
    }
}
