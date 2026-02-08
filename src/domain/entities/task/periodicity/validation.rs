use std::collections::HashSet;
use chrono::{DateTime, Utc};
use super::{
    DayConstraint, MonthConstraint, MonthWeekPosition, Periodicity, PeriodicityConstraints,
    SpecialPattern, WeekConstraint, YearConstraint,
    OccurrenceTimingSettings, RepTimingSettings, RepetitionUnit,
};

// ========================================================================
// VALIDATION ERRORS
// Comprehensive error types for all validation scenarios
// ========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Invalid value for a specific field
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },
    
    /// Missing required field
    MissingRequired {
        field: String,
        reason: String,
    },
    
    /// Constraints incompatible with repetition unit
    IncompatibleConstraint {
        rep_unit: RepetitionUnit,
        constraint_type: String,
        reason: String,
    },
    
    /// Constraints conflict with each other
    ConflictingConstraints {
        constraint1: String,
        constraint2: String,
        reason: String,
    },
    
    /// Duplicate values where uniqueness is required
    DuplicateValues {
        field: String,
        reason: String,
    },
    
    /// Empty collection where values are required
    EmptyCollection {
        field: String,
        reason: String,
    },
    
    /// Value out of acceptable range
    OutOfRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },
    
    /// Timeframe validation errors
    InvalidTimeframe {
        reason: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidValue { field, value, reason } => {
                write!(f, "Invalid value for {}: '{}' - {}", field, value, reason)
            }
            ValidationError::MissingRequired { field, reason } => {
                write!(f, "Missing required field {}: {}", field, reason)
            }
            ValidationError::IncompatibleConstraint { rep_unit, constraint_type, reason } => {
                write!(f, "Constraint {} incompatible with {:?} repetition: {}", 
                    constraint_type, rep_unit, reason)
            }
            ValidationError::ConflictingConstraints { constraint1, constraint2, reason } => {
                write!(f, "Constraints {} and {} conflict: {}", constraint1, constraint2, reason)
            }
            ValidationError::DuplicateValues { field, reason } => {
                write!(f, "Duplicate values in {}: {}", field, reason)
            }
            ValidationError::EmptyCollection { field, reason } => {
                write!(f, "Empty collection for {}: {}", field, reason)
            }
            ValidationError::OutOfRange { field, value, min, max } => {
                write!(f, "{} value {} out of range [{}, {}]", field, value, min, max)
            }
            ValidationError::InvalidTimeframe { reason } => {
                write!(f, "Invalid timeframe: {}", reason)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

// ========================================================================
// MAIN VALIDATION FUNCTION
// Entry point for validating entire Periodicity struct
// ========================================================================

pub fn validate_periodicity(periodicity: &Periodicity) -> Result<(), ValidationError> {
    // 1. Validate special patterns first (short-circuit if present)
    if let Some(pattern) = &periodicity.special_pattern {
        return validate_special_pattern(periodicity, pattern);
    }
    
    // 2. Validate repetition unit and count
    validate_repetition(periodicity)?;
    
    // 3. Validate individual constraints
    validate_constraints(&periodicity.constraints)?;
    
    // 4. Validate constraint compatibility with repetition unit
    validate_constraint_compatibility(periodicity)?;
    
    // 5. Validate timeframe if present
    validate_timeframe(&periodicity.timeframe)?;
    
    // 6. Validate occurrence settings if present
    validate_occurrence_settings(&periodicity.occurrence_settings, periodicity.rep_per_unit)?;
    
    Ok(())
}

// ========================================================================
// REPETITION VALIDATION
// ========================================================================

fn validate_repetition(periodicity: &Periodicity) -> Result<(), ValidationError> {
    match periodicity.rep_unit {
        RepetitionUnit::None => {
            // rep_per_unit must be None
            if periodicity.rep_per_unit.is_some() {
                return Err(ValidationError::InvalidValue {
                    field: "rep_per_unit".into(),
                    value: periodicity.rep_per_unit.unwrap().to_string(),
                    reason: "Must be None when rep_unit is None".into(),
                });
            }
        }
        _ => {
            // rep_per_unit must be Some and > 0
            match periodicity.rep_per_unit {
                None => {
                    return Err(ValidationError::MissingRequired {
                        field: "rep_per_unit".into(),
                        reason: format!("Required when rep_unit is {:?}", periodicity.rep_unit),
                    });
                }
                Some(0) => {
                    return Err(ValidationError::InvalidValue {
                        field: "rep_per_unit".into(),
                        value: "0".into(),
                        reason: "Must be at least 1".into(),
                    });
                }
                Some(count) => {
                    // Validate practical limits per unit
                    let max = match periodicity.rep_unit {
                        RepetitionUnit::Day => 100,   // Max 100 times per day
                        RepetitionUnit::Week => 50,   // Max 50 times per week
                        RepetitionUnit::Month => 100, // Max 100 times per month
                        RepetitionUnit::Year => 255,  // Max 366 times per year
                        RepetitionUnit::None => unreachable!(),
                    };
                    
                    if count > max {
                        return Err(ValidationError::OutOfRange {
                            field: "rep_per_unit".into(),
                            value: count.to_string(),
                            min: "1".into(),
                            max: max.to_string(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(())
}

// ========================================================================
// CONSTRAINTS VALIDATION
// Validate individual constraint configurations
// ========================================================================

fn validate_constraints(constraints: &PeriodicityConstraints) -> Result<(), ValidationError> {
    if let Some(day) = &constraints.day_constraint {
        validate_day_constraint(day)?;
    }
    
    if let Some(week) = &constraints.week_constraint {
        validate_week_constraint(week)?;
    }
    
    if let Some(month) = &constraints.month_constraint {
        validate_month_constraint(month)?;
    }
    
    if let Some(year) = &constraints.year_constraint {
        validate_year_constraint(year)?;
    }
    
    Ok(())
}

fn validate_day_constraint(constraint: &DayConstraint) -> Result<(), ValidationError> {
    match constraint {
        DayConstraint::EveryDay => Ok(()),
        
        DayConstraint::EveryNDays(n) => {
            if *n == 0 {
                return Err(ValidationError::InvalidValue {
                    field: "EveryNDays".into(),
                    value: "0".into(),
                    reason: "Must be at least 1".into(),
                });
            }
            if *n > 366 {
                return Err(ValidationError::OutOfRange {
                    field: "EveryNDays".into(),
                    value: n.to_string(),
                    min: "1".into(),
                    max: "366".into(),
                });
            }
            Ok(())
        }
        
        DayConstraint::SpecificDaysWeek(weekdays) => {
            if weekdays.is_empty() {
                return Err(ValidationError::EmptyCollection {
                    field: "SpecificDaysWeek".into(),
                    reason: "Must contain at least one weekday".into(),
                });
            }
            if weekdays.len() > 7 {
                return Err(ValidationError::OutOfRange {
                    field: "SpecificDaysWeek".into(),
                    value: weekdays.len().to_string(),
                    min: "1".into(),
                    max: "7".into(),
                });
            }
            // Check for duplicates
            let unique: HashSet<_> = weekdays.iter().collect();
            if unique.len() != weekdays.len() {
                return Err(ValidationError::DuplicateValues {
                    field: "SpecificDaysWeek".into(),
                    reason: "Weekdays must be unique".into(),
                });
            }
            Ok(())
        }
        
        DayConstraint::SpecificDaysMonthFromFirst(days) => {
            validate_month_days(days, "SpecificDaysMonthFromFirst")
        }
        
        DayConstraint::SpecificDaysMonthFromLast(days) => {
            validate_month_days(days, "SpecificDaysMonthFromLast")
        }
        
        DayConstraint::SpecificNthWeekdaysMonth(patterns) => {
            if patterns.is_empty() {
                return Err(ValidationError::EmptyCollection {
                    field: "SpecificNthWeekdaysMonth".into(),
                    reason: "Must contain at least one pattern".into(),
                });
            }
            if patterns.len() > 20 {
                return Err(ValidationError::OutOfRange {
                    field: "SpecificNthWeekdaysMonth".into(),
                    value: patterns.len().to_string(),
                    min: "1".into(),
                    max: "20".into(),
                });
            }
            // Validate each position
            for pattern in patterns {
                pattern.position.validate()?;
            }
            // Check for duplicates
            let unique: HashSet<_> = patterns.iter()
                .map(|p| (p.weekday, match p.position {
                    MonthWeekPosition::FromFirst(n) => (true, n),
                    MonthWeekPosition::FromLast(n) => (false, n),
                }))
                .collect();
            if unique.len() != patterns.len() {
                return Err(ValidationError::DuplicateValues {
                    field: "SpecificNthWeekdaysMonth".into(),
                    reason: "Patterns must be unique".into(),
                });
            }
            Ok(())
        }
    }
}

fn validate_month_days(days: &[u8], field_name: &str) -> Result<(), ValidationError> {
    if days.is_empty() {
        return Err(ValidationError::EmptyCollection {
            field: field_name.into(),
            reason: "Must contain at least one day".into(),
        });
    }
    if days.len() > 31 {
        return Err(ValidationError::OutOfRange {
            field: field_name.into(),
            value: days.len().to_string(),
            min: "1".into(),
            max: "31".into(),
        });
    }
    for &day in days {
        if day > 30 {
            return Err(ValidationError::OutOfRange {
                field: field_name.into(),
                value: day.to_string(),
                min: "0".into(),
                max: "30".into(),
            });
        }
    }
    // Check for duplicates
    let unique: HashSet<_> = days.iter().collect();
    if unique.len() != days.len() {
        return Err(ValidationError::DuplicateValues {
            field: field_name.into(),
            reason: "Days must be unique".into(),
        });
    }
    Ok(())
}

fn validate_week_constraint(constraint: &WeekConstraint) -> Result<(), ValidationError> {
    match constraint {
        WeekConstraint::EveryWeek => Ok(()),
        
        WeekConstraint::EveryNWeeks(n) => {
            if *n == 0 {
                return Err(ValidationError::InvalidValue {
                    field: "EveryNWeeks".into(),
                    value: "0".into(),
                    reason: "Must be at least 1".into(),
                });
            }
            if *n > 52 {
                return Err(ValidationError::OutOfRange {
                    field: "EveryNWeeks".into(),
                    value: n.to_string(),
                    min: "1".into(),
                    max: "52".into(),
                });
            }
            Ok(())
        }
        
        WeekConstraint::SpecificWeeksOfMonthFromFirst(weeks) => {
            validate_weeks_of_month(weeks, "SpecificWeeksOfMonthFromFirst")
        }
        
        WeekConstraint::SpecificWeeksOfMonthFromLast(weeks) => {
            validate_weeks_of_month(weeks, "SpecificWeeksOfMonthFromLast")
        }
    }
}

fn validate_weeks_of_month(weeks: &[u8], field_name: &str) -> Result<(), ValidationError> {
    if weeks.is_empty() {
        return Err(ValidationError::EmptyCollection {
            field: field_name.into(),
            reason: "Must contain at least one week".into(),
        });
    }
    if weeks.len() > 5 {
        return Err(ValidationError::OutOfRange {
            field: field_name.into(),
            value: weeks.len().to_string(),
            min: "1".into(),
            max: "5".into(),
        });
    }
    for &week in weeks {
        if week > 4 {
            return Err(ValidationError::OutOfRange {
                field: field_name.into(),
                value: week.to_string(),
                min: "0".into(),
                max: "4".into(),
            });
        }
    }
    // Check for duplicates
    let unique: HashSet<_> = weeks.iter().collect();
    if unique.len() != weeks.len() {
        return Err(ValidationError::DuplicateValues {
            field: field_name.into(),
            reason: "Weeks must be unique".into(),
        });
    }
    Ok(())
}

fn validate_month_constraint(constraint: &MonthConstraint) -> Result<(), ValidationError> {
    match constraint {
        MonthConstraint::EveryMonth => Ok(()),
        
        MonthConstraint::EveryNMonths(n) => {
            if *n == 0 {
                return Err(ValidationError::InvalidValue {
                    field: "EveryNMonths".into(),
                    value: "0".into(),
                    reason: "Must be at least 1".into(),
                });
            }
            if *n > 12 {
                return Err(ValidationError::OutOfRange {
                    field: "EveryNMonths".into(),
                    value: n.to_string(),
                    min: "1".into(),
                    max: "12".into(),
                });
            }
            Ok(())
        }
        
        MonthConstraint::SpecificMonths(months) => {
            if months.is_empty() {
                return Err(ValidationError::EmptyCollection {
                    field: "SpecificMonths".into(),
                    reason: "Must contain at least one month".into(),
                });
            }
            if months.len() > 12 {
                return Err(ValidationError::OutOfRange {
                    field: "SpecificMonths".into(),
                    value: months.len().to_string(),
                    min: "1".into(),
                    max: "12".into(),
                });
            }
            // Check for duplicates
            let unique: HashSet<_> = months.iter().collect();
            if unique.len() != months.len() {
                return Err(ValidationError::DuplicateValues {
                    field: "SpecificMonths".into(),
                    reason: "Months must be unique".into(),
                });
            }
            Ok(())
        }
    }
}

fn validate_year_constraint(constraint: &YearConstraint) -> Result<(), ValidationError> {
    match constraint {
        YearConstraint::EveryYear => Ok(()),
        
        YearConstraint::EveryNYears(n) => {
            if *n == 0 {
                return Err(ValidationError::InvalidValue {
                    field: "EveryNYears".into(),
                    value: "0".into(),
                    reason: "Must be at least 1".into(),
                });
            }
            if *n > 100 {
                return Err(ValidationError::OutOfRange {
                    field: "EveryNYears".into(),
                    value: n.to_string(),
                    min: "1".into(),
                    max: "100".into(),
                });
            }
            Ok(())
        }
        
        YearConstraint::SpecificYears(years) => {
            if years.is_empty() {
                return Err(ValidationError::EmptyCollection {
                    field: "SpecificYears".into(),
                    reason: "Must contain at least one year".into(),
                });
            }
            if years.len() > 100 {
                return Err(ValidationError::OutOfRange {
                    field: "SpecificYears".into(),
                    value: years.len().to_string(),
                    min: "1".into(),
                    max: "100".into(),
                });
            }
            // Check for duplicates
            let unique: HashSet<_> = years.iter().collect();
            if unique.len() != years.len() {
                return Err(ValidationError::DuplicateValues {
                    field: "SpecificYears".into(),
                    reason: "Years must be unique".into(),
                });
            }
            // Validate year range (1900-2200)
            for &year in years {
                if year < 1900 || year > 2200 {
                    return Err(ValidationError::OutOfRange {
                        field: "SpecificYears".into(),
                        value: year.to_string(),
                        min: "1900".into(),
                        max: "2200".into(),
                    });
                }
            }
            Ok(())
        }
    }
}

// ========================================================================
// CONSTRAINT COMPATIBILITY VALIDATION
// Ensures constraints make sense with the repetition unit
// ========================================================================

fn validate_constraint_compatibility(periodicity: &Periodicity) -> Result<(), ValidationError> {
    let constraints = &periodicity.constraints;
    
    // Special patterns must have RepetitionUnit::None
    if periodicity.special_pattern.is_some() && periodicity.rep_unit != RepetitionUnit::None {
        return Err(ValidationError::IncompatibleConstraint {
            rep_unit: periodicity.rep_unit,
            constraint_type: "special_pattern".into(),
            reason: "Special patterns require rep_unit to be None".into(),
        });
    }
    
    match periodicity.rep_unit {
        RepetitionUnit::None => {
            // Must have a special pattern
            if periodicity.special_pattern.is_none() {
                return Err(ValidationError::MissingRequired {
                    field: "special_pattern".into(),
                    reason: "Required when rep_unit is None".into(),
                });
            }
            Ok(())
        }
        
        RepetitionUnit::Day => {
            // Day repetition is compatible with all constraints
            // No specific incompatibilities
            Ok(())
        }
        
        RepetitionUnit::Week => {
            // Week-level repetition shouldn't have EveryNDays
            if let Some(DayConstraint::EveryNDays(_)) = constraints.day_constraint {
                return Err(ValidationError::IncompatibleConstraint {
                    rep_unit: periodicity.rep_unit,
                    constraint_type: "EveryNDays".into(),
                    reason: "Use Week repetition unit instead".into(),
                });
            }
            Ok(())
        }
        
        RepetitionUnit::Month => {
            // Month-level repetition shouldn't have EveryNWeeks
            if let Some(WeekConstraint::EveryNWeeks(_)) = constraints.week_constraint {
                return Err(ValidationError::IncompatibleConstraint {
                    rep_unit: periodicity.rep_unit,
                    constraint_type: "EveryNWeeks".into(),
                    reason: "Use Month repetition unit instead".into(),
                });
            }
            Ok(())
        }
        
        RepetitionUnit::Year => {
            // Year-level repetition shouldn't have EveryNMonths
            if let Some(MonthConstraint::EveryNMonths(_)) = constraints.month_constraint {
                return Err(ValidationError::IncompatibleConstraint {
                    rep_unit: periodicity.rep_unit,
                    constraint_type: "EveryNMonths".into(),
                    reason: "Use Year repetition unit instead".into(),
                });
            }
            Ok(())
        }
    }
}

// ========================================================================
// SPECIAL PATTERN VALIDATION
// ========================================================================

fn validate_special_pattern(
    periodicity: &Periodicity,
    pattern: &SpecialPattern,
) -> Result<(), ValidationError> {
    // Must have RepetitionUnit::None
    if periodicity.rep_unit != RepetitionUnit::None {
        return Err(ValidationError::IncompatibleConstraint {
            rep_unit: periodicity.rep_unit,
            constraint_type: "special_pattern".into(),
            reason: "Special patterns require rep_unit to be None".into(),
        });
    }
    
    // Must not have rep_per_unit
    if periodicity.rep_per_unit.is_some() {
        return Err(ValidationError::InvalidValue {
            field: "rep_per_unit".into(),
            value: periodicity.rep_per_unit.unwrap().to_string(),
            reason: "Must be None for special patterns".into(),
        });
    }
    
    // Must not have any constraints
    let constraints = &periodicity.constraints;
    if constraints.day_constraint.is_some()
        || constraints.week_constraint.is_some()
        || constraints.month_constraint.is_some()
        || constraints.year_constraint.is_some()
    {
        return Err(ValidationError::ConflictingConstraints {
            constraint1: "special_pattern".into(),
            constraint2: "regular constraints".into(),
            reason: "Special patterns cannot be combined with regular constraints".into(),
        });
    }
    
    // Validate the pattern itself
    match pattern {
        SpecialPattern::Custom(custom) => {
            if custom.dates.is_empty() {
                return Err(ValidationError::EmptyCollection {
                    field: "CustomDates".into(),
                    reason: "Must contain at least one date".into(),
                });
            }
            // Dates should be sorted and unique (enforced by constructor)
        }
        SpecialPattern::Unique(_) => {
            // Always valid
        }
    }
    
    Ok(())
}

// ========================================================================
// TIMEFRAME VALIDATION
// ========================================================================

fn validate_timeframe(
    timeframe: &Option<(DateTime<Utc>, DateTime<Utc>)>,
) -> Result<(), ValidationError> {
    if let Some((start, end)) = timeframe {
        if start >= end {
            return Err(ValidationError::InvalidTimeframe {
                reason: format!("Start ({}) must be before end ({})", start, end),
            });
        }
    }
    Ok(())
}

// ========================================================================
// OCCURRENCE SETTINGS VALIDATION
// ========================================================================

pub fn validate_occurrence_settings(
    settings: &Option<OccurrenceTimingSettings>,
    rep_per_unit: Option<u8>,
) -> Result<(), ValidationError> {
    let Some(settings) = settings else {
        return Ok(());
    };
    
    // Validate duration (max 24 hours = 1440 minutes)
    if let Some(duration) = settings.duration {
        if duration == 0 {
            return Err(ValidationError::InvalidValue {
                field: "duration".into(),
                value: "0".into(),
                reason: "Duration must be at least 1 minute".into(),
            });
        }
        if duration > 1440 {
            return Err(ValidationError::OutOfRange {
                field: "duration".into(),
                value: duration.to_string(),
                min: "1".into(),
                max: "1440".into(),
            });
        }
    }
    
    // Validate not_before < best_before if both present
    if let (Some(not_before), Some(best_before)) = (settings.not_before, settings.best_before) {
        if not_before >= best_before {
            return Err(ValidationError::InvalidValue {
                field: "not_before/best_before".into(),
                value: format!("{}/{}", not_before, best_before),
                reason: "not_before must be earlier than best_before".into(),
            });
        }
    }
    
    // Validate rep_timing_settings if present
    if let Some(rep_settings) = &settings.rep_timing_settings {
        validate_rep_timing_settings(rep_settings, rep_per_unit)?;
    }
    
    Ok(())
}

fn validate_rep_timing_settings(
    rep_settings: &Vec<RepTimingSettings>,
    rep_per_unit: Option<u8>,
) -> Result<(), ValidationError> {
    // Must not be empty
    if rep_settings.is_empty() {
        return Err(ValidationError::EmptyCollection {
            field: "rep_timing_settings".into(),
            reason: "If specified, must contain at least one RepTimingSettings".into(),
        });
    }
    
    // Check for duplicate rep_index values
    let mut seen_indices = HashSet::new();
    for rep in rep_settings {
        if !seen_indices.insert(rep.rep_index) {
            return Err(ValidationError::DuplicateValues {
                field: "rep_timing_settings.rep_index".into(),
                reason: format!("Duplicate rep_index: {}", rep.rep_index),
            });
        }
    }
    
    // Validate each RepTimingSettings
    for rep in rep_settings {
        // Validate rep_index is within bounds if rep_per_unit is known
        if let Some(count) = rep_per_unit {
            if rep.rep_index >= count {
                return Err(ValidationError::OutOfRange {
                    field: "rep_timing_settings.rep_index".into(),
                    value: rep.rep_index.to_string(),
                    min: "0".into(),
                    max: (count - 1).to_string(),
                });
            }
        }
        
        // Validate not_before < best_before if both present
        if let (Some(not_before), Some(best_before)) = (rep.not_before, rep.best_before) {
            if not_before >= best_before {
                return Err(ValidationError::InvalidValue {
                    field: format!("rep_timing_settings[{}]", rep.rep_index),
                    value: format!("not_before={}, best_before={}", not_before, best_before),
                    reason: "not_before must be earlier than best_before".into(),
                });
            }
        }
    }
    
    Ok(())
}

// ========================================================================
// UNIT TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveTime, Utc, Weekday};
    use crate::domain::entities::task::periodicity::UniqueDate;
    
    #[test]
    fn test_validate_repetition_none_requires_none_count() {
        let periodicity = Periodicity {
            rep_unit: RepetitionUnit::None,
            rep_per_unit: Some(1),
            occurrence_settings: None,
            constraints: PeriodicityConstraints::default(),
            timeframe: None,
            special_pattern: Some(SpecialPattern::Unique(UniqueDate {
                date: Utc::now(),
            })),
            reference_date: None,
        };
        
        assert!(periodicity.validate().is_err());
    }
    
    #[test]
    fn test_validate_repetition_non_none_requires_count() {
        let periodicity = Periodicity {
            rep_unit: RepetitionUnit::Day,
            rep_per_unit: None,
            occurrence_settings: None,
            constraints: PeriodicityConstraints::default(),
            timeframe: None,
            special_pattern: None,
            reference_date: None,
        };
        
        assert!(periodicity.validate().is_err());
    }
    
    #[test]
    fn test_validate_day_constraint_empty_weekdays() {
        let constraint = DayConstraint::SpecificDaysWeek(vec![]);
        assert!(validate_day_constraint(&constraint).is_err());
    }
    
    #[test]
    fn test_validate_day_constraint_duplicate_weekdays() {
        let constraint = DayConstraint::SpecificDaysWeek(vec![
            Weekday::Mon,
            Weekday::Mon,
        ]);
        assert!(validate_day_constraint(&constraint).is_err());
    }
    
    #[test]
    fn test_validate_timeframe_start_after_end() {
        let now = Utc::now();
        let past = now - chrono::Duration::days(1);
        let timeframe = Some((now, past));
        
        assert!(validate_timeframe(&timeframe).is_err());
    }
    
    // ========================================================================
    // OCCURRENCE SETTINGS TESTS
    // ========================================================================
    
    #[test]
    fn test_validate_occurrence_settings_none() {
        // None is always valid
        assert!(validate_occurrence_settings(&None, Some(3)).is_ok());
    }
    
    #[test]
    fn test_validate_occurrence_settings_valid() {
        let settings = OccurrenceTimingSettings {
            duration: Some(30),
            not_before: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            best_before: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
            rep_timing_settings: None,
        };
        
        assert!(validate_occurrence_settings(&Some(settings), Some(3)).is_ok());
    }
    
    #[test]
    fn test_validate_occurrence_settings_zero_duration() {
        let settings = OccurrenceTimingSettings {
            duration: Some(0),
            not_before: None,
            best_before: None,
            rep_timing_settings: None,
        };
        
        let result = validate_occurrence_settings(&Some(settings), Some(3));
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::InvalidValue { field, value, .. } => {
                assert_eq!(field, "duration");
                assert_eq!(value, "0");
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }
    
    #[test]
    fn test_validate_occurrence_settings_duration_too_large() {
        let settings = OccurrenceTimingSettings {
            duration: Some(1441), // > 24 hours
            not_before: None,
            best_before: None,
            rep_timing_settings: None,
        };
        
        let result = validate_occurrence_settings(&Some(settings), Some(3));
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::OutOfRange { field, value, max, .. } => {
                assert_eq!(field, "duration");
                assert_eq!(value, "1441");
                assert_eq!(max, "1440");
            }
            _ => panic!("Expected OutOfRange error"),
        }
    }
    
    #[test]
    fn test_validate_occurrence_settings_not_before_after_best_before() {
        let settings = OccurrenceTimingSettings {
            duration: Some(30),
            not_before: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
            best_before: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()), // Earlier!
            rep_timing_settings: None,
        };
        
        let result = validate_occurrence_settings(&Some(settings), Some(3));
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::InvalidValue { field, reason, .. } => {
                assert_eq!(field, "not_before/best_before");
                assert!(reason.contains("not_before must be earlier"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }
    
    #[test]
    fn test_validate_occurrence_settings_not_before_equals_best_before() {
        let time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let settings = OccurrenceTimingSettings {
            duration: Some(30),
            not_before: Some(time),
            best_before: Some(time), // Same time
            rep_timing_settings: None,
        };
        
        let result = validate_occurrence_settings(&Some(settings), Some(3));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_rep_timing_settings_empty() {
        let settings = OccurrenceTimingSettings {
            duration: Some(30),
            not_before: None,
            best_before: None,
            rep_timing_settings: Some(vec![]), // Empty!
        };
        
        let result = validate_occurrence_settings(&Some(settings), Some(3));
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::EmptyCollection { field, .. } => {
                assert_eq!(field, "rep_timing_settings");
            }
            _ => panic!("Expected EmptyCollection error"),
        }
    }
    
    #[test]
    fn test_validate_rep_timing_settings_duplicate_index() {
        let settings = OccurrenceTimingSettings {
            duration: Some(30),
            not_before: None,
            best_before: None,
            rep_timing_settings: Some(vec![
                RepTimingSettings {
                    rep_index: 0,
                    not_before: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                    best_before: None,
                },
                RepTimingSettings {
                    rep_index: 0, // Duplicate!
                    not_before: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                    best_before: None,
                },
            ]),
        };
        
        let result = validate_occurrence_settings(&Some(settings), Some(3));
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::DuplicateValues { field, .. } => {
                assert_eq!(field, "rep_timing_settings.rep_index");
            }
            _ => panic!("Expected DuplicateValues error"),
        }
    }
    
    #[test]
    fn test_validate_rep_timing_settings_index_out_of_range() {
        let settings = OccurrenceTimingSettings {
            duration: Some(30),
            not_before: None,
            best_before: None,
            rep_timing_settings: Some(vec![
                RepTimingSettings {
                    rep_index: 3, // Out of range for rep_per_unit=3 (valid: 0, 1, 2)
                    not_before: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                    best_before: None,
                },
            ]),
        };
        
        let result = validate_occurrence_settings(&Some(settings), Some(3));
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::OutOfRange { field, value, max, .. } => {
                assert_eq!(field, "rep_timing_settings.rep_index");
                assert_eq!(value, "3");
                assert_eq!(max, "2");
            }
            _ => panic!("Expected OutOfRange error"),
        }
    }
    
    #[test]
    fn test_validate_rep_timing_settings_valid() {
        let settings = OccurrenceTimingSettings {
            duration: Some(30),
            not_before: None,
            best_before: None,
            rep_timing_settings: Some(vec![
                RepTimingSettings {
                    rep_index: 0,
                    not_before: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                    best_before: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
                },
                RepTimingSettings {
                    rep_index: 1,
                    not_before: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                    best_before: Some(NaiveTime::from_hms_opt(14, 0, 0).unwrap()),
                },
                RepTimingSettings {
                    rep_index: 2,
                    not_before: Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
                    best_before: Some(NaiveTime::from_hms_opt(20, 0, 0).unwrap()),
                },
            ]),
        };
        
        assert!(validate_occurrence_settings(&Some(settings), Some(3)).is_ok());
    }
    
    #[test]
    fn test_validate_rep_timing_settings_time_order_invalid() {
        let settings = OccurrenceTimingSettings {
            duration: Some(30),
            not_before: None,
            best_before: None,
            rep_timing_settings: Some(vec![
                RepTimingSettings {
                    rep_index: 0,
                    not_before: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
                    best_before: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()), // Invalid!
                },
            ]),
        };
        
        let result = validate_occurrence_settings(&Some(settings), Some(3));
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::InvalidValue { field, reason, .. } => {
                assert!(field.contains("rep_timing_settings"));
                assert!(reason.contains("not_before must be earlier"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }
    
    #[test]
    fn test_validate_rep_timing_settings_no_rep_per_unit() {
        // When rep_per_unit is None, we can't validate index bounds
        // But other validations still apply
        let settings = OccurrenceTimingSettings {
            duration: Some(30),
            not_before: None,
            best_before: None,
            rep_timing_settings: Some(vec![
                RepTimingSettings {
                    rep_index: 10, // Large index, but can't validate without rep_per_unit
                    not_before: Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                    best_before: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
                },
            ]),
        };
        
        // Should pass because we don't know the valid range
        assert!(validate_occurrence_settings(&Some(settings), None).is_ok());
    }
}
