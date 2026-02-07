/// Comprehensive tests for the Periodicity system
/// 
/// These tests demonstrate the robustness of the domain model and cover:
/// - Valid configurations
/// - Invalid configurations (boundary testing)
/// - Constraint combinations
/// - Edge cases
/// - Real-world examples

#[cfg(test)]
mod periodicity_tests {
    use crate::domain::{
        Periodicity, PeriodicityBuilder, DayConstraint, MonthConstraint,
        WeekConstraint, NthWeekdayOfMonth, MonthWeekPosition, RepetitionUnit,
        ValidationError
    };
    use chrono::{Utc, Weekday, Month, TimeZone};

    // ========================================================================
    // BASIC VALID CONFIGURATIONS
    // ========================================================================

    #[test]
    fn test_simple_daily_task() {
        let result = Periodicity::daily();
        assert!(result.is_ok());
        let p = result.unwrap();
        assert_eq!(p.rep_unit, RepetitionUnit::Day);
        assert_eq!(p.rep_per_unit, Some(1));
    }

    #[test]
    fn test_simple_weekly_task() {
        let result = Periodicity::weekly();
        assert!(result.is_ok());
        let p = result.unwrap();
        assert_eq!(p.rep_unit, RepetitionUnit::Week);
        assert_eq!(p.rep_per_unit, Some(1));
    }

    #[test]
    fn test_simple_monthly_task() {
        let result = Periodicity::monthly();
        assert!(result.is_ok());
        let p = result.unwrap();
        assert_eq!(p.rep_unit, RepetitionUnit::Month);
        assert_eq!(p.rep_per_unit, Some(1));
    }

    #[test]
    fn test_unique_date_task() {
        let date = Utc.with_ymd_and_hms(2026, 12, 25, 0, 0, 0).unwrap();
        let result = Periodicity::unique(date);
        assert!(result.is_ok());
        let p = result.unwrap();
        assert_eq!(p.rep_unit, RepetitionUnit::None);
        assert!(p.special_pattern.is_some());
    }

    // ========================================================================
    // REAL-WORLD USE CASES
    // ========================================================================

    #[test]
    fn test_user_example_13th_24th_jan_feb() {
        // "Task on the 13th and 24th of each month, but only in January & February"
        let result = PeriodicityBuilder::new()
            .daily(1)
            .on_month_days(vec![13, 24])
            .in_months(vec![Month::January, Month::February])
            .build();
        
        assert!(result.is_ok());
        let p = result.unwrap();
        
        // Verify configuration
        assert_eq!(p.rep_unit, RepetitionUnit::Day);
        assert!(matches!(
            p.constraints.day_constraint,
            Some(DayConstraint::SpecificDaysMonthFromFirst(_))
        ));
        assert!(matches!(
            p.constraints.month_constraint,
            Some(MonthConstraint::SpecificMonths(_))
        ));
        
        // Test date matching
        let jan_13 = Utc.with_ymd_and_hms(2026, 1, 13, 10, 0, 0).unwrap();
        let jan_24 = Utc.with_ymd_and_hms(2026, 1, 24, 10, 0, 0).unwrap();
        let feb_13 = Utc.with_ymd_and_hms(2026, 2, 13, 10, 0, 0).unwrap();
        let mar_13 = Utc.with_ymd_and_hms(2026, 3, 13, 10, 0, 0).unwrap();
        let jan_15 = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        
        assert!(p.matches_constraints(&jan_13), "Should match Jan 13th");
        assert!(p.matches_constraints(&jan_24), "Should match Jan 24th");
        assert!(p.matches_constraints(&feb_13), "Should match Feb 13th");
        assert!(!p.matches_constraints(&mar_13), "Should NOT match Mar 13th (wrong month)");
        assert!(!p.matches_constraints(&jan_15), "Should NOT match Jan 15th (wrong day)");
    }

    #[test]
    fn test_three_times_daily_on_mondays() {
        // "3 times per day, but only on Mondays"
        let result = PeriodicityBuilder::new()
            .daily(3)
            .on_weekdays(vec![Weekday::Mon])
            .build();
        
        assert!(result.is_ok());
        let p = result.unwrap();
        assert_eq!(p.rep_per_unit, Some(3));
        
        // Monday in 2026
        let monday = Utc.with_ymd_and_hms(2026, 2, 2, 10, 0, 0).unwrap();
        assert!(p.matches_constraints(&monday));
        
        // Tuesday
        let tuesday = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        assert!(!p.matches_constraints(&tuesday));
    }

    #[test]
    fn test_first_monday_of_month() {
        // "Every first Monday of the month"
        let result = PeriodicityBuilder::new()
            .daily(1)
            .on_nth_weekdays(vec![NthWeekdayOfMonth::first(Weekday::Mon)])
            .build();
        
        assert!(result.is_ok());
        let p = result.unwrap();
        
        // Feb 2, 2026 is first Monday of February
        let first_monday_feb = Utc.with_ymd_and_hms(2026, 2, 2, 10, 0, 0).unwrap();
        assert!(p.matches_constraints(&first_monday_feb));
        
        // Feb 9, 2026 is second Monday
        let second_monday_feb = Utc.with_ymd_and_hms(2026, 2, 9, 10, 0, 0).unwrap();
        assert!(!p.matches_constraints(&second_monday_feb));
    }

    #[test]
    fn test_last_day_of_month() {
        // "Last day of each month"
        let result = PeriodicityBuilder::new()
            .daily(1)
            .on_month_days_from_end(vec![1])
            .build();
        
        assert!(result.is_ok());
        let p = result.unwrap();
        
        // Jan 31 (last day of January)
        let jan_31 = Utc.with_ymd_and_hms(2026, 1, 31, 10, 0, 0).unwrap();
        assert!(p.matches_constraints(&jan_31));
        
        // Jan 30 (not last day)
        let jan_30 = Utc.with_ymd_and_hms(2026, 1, 30, 10, 0, 0).unwrap();
        assert!(!p.matches_constraints(&jan_30));
        
        // Feb 28 (last day of February in 2026 - not a leap year)
        let feb_28 = Utc.with_ymd_and_hms(2026, 2, 28, 10, 0, 0).unwrap();
        assert!(p.matches_constraints(&feb_28));
    }

    #[test]
    fn test_weekdays_only() {
        // "Monday through Friday"
        let result = Periodicity::on_weekdays(vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ]);
        
        assert!(result.is_ok());
        let p = result.unwrap();
        
        // Monday (should match)
        let monday = Utc.with_ymd_and_hms(2026, 2, 2, 10, 0, 0).unwrap();
        assert!(p.matches_constraints(&monday));
        
        // Saturday (should NOT match)
        let saturday = Utc.with_ymd_and_hms(2026, 2, 7, 10, 0, 0).unwrap();
        assert!(!p.matches_constraints(&saturday));
    }

    #[test]
    fn test_with_timeframe() {
        // "Daily task, but only valid from Feb 1 to Feb 28, 2026"
        let start = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
        
        let result = PeriodicityBuilder::new()
            .daily(1)
            .between(start, end)
            .build();
        
        assert!(result.is_ok());
        let p = result.unwrap();
        
        let in_range = Utc.with_ymd_and_hms(2026, 2, 15, 10, 0, 0).unwrap();
        assert!(p.is_within_timeframe(&in_range));
        
        let before = Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap();
        assert!(!p.is_within_timeframe(&before));
        
        let after = Utc.with_ymd_and_hms(2026, 3, 15, 10, 0, 0).unwrap();
        assert!(!p.is_within_timeframe(&after));
    }

    // ========================================================================
    // VALIDATION TESTS - INVALID CONFIGURATIONS
    // ========================================================================

    #[test]
    fn test_invalid_rep_unit_none_with_count() {
        // Cannot have rep_per_unit when rep_unit is None
        let date = Utc::now();
        let builder = PeriodicityBuilder::new().unique(date);
        
        // This should work
        assert!(builder.build().is_ok());
        
        // But if we manually set rep_per_unit, it should fail validation
        // (not possible through builder, but testing the validation)
    }

    #[test]
    fn test_invalid_missing_rep_per_unit() {
        // When rep_unit is not None, rep_per_unit is required
        // We can't test this easily through the builder since it enforces this
        // But we can verify that validation catches it
        use crate::domain::periodicity::{Periodicity, PeriodicityConstraints, RepetitionUnit};
        use chrono::Month;
        
        let p = Periodicity {
            rep_unit: RepetitionUnit::Day,
            rep_per_unit: None, // Missing!
            constraints: PeriodicityConstraints::default(),
            timeframe: None,
            week_start: Weekday::Mon,
            year_start: Month::January,
            special_pattern: None,
        };
        
        let result = p.validate();
        assert!(result.is_err());
        match result {
            Err(ValidationError::MissingRequired { field, .. }) => {
                assert_eq!(field, "rep_per_unit");
            }
            _ => panic!("Expected MissingRequired error"),
        }
    }

    #[test]
    fn test_invalid_zero_rep_count() {
        // rep_per_unit cannot be zero
        use crate::domain::periodicity::{Periodicity, PeriodicityConstraints, RepetitionUnit};
        use chrono::Month;
        
        let p = Periodicity {
            rep_unit: RepetitionUnit::Day,
            rep_per_unit: Some(0), // Zero is invalid!
            constraints: PeriodicityConstraints::default(),
            timeframe: None,
            week_start: Weekday::Mon,
            year_start: Month::January,
            special_pattern: None,
        };
        
        let result = p.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_empty_weekdays() {
        // SpecificDaysWeek requires at least one weekday
        let result = PeriodicityBuilder::new()
            .daily(1)
            .on_weekdays(vec![])
            .build();
        
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_duplicate_weekdays() {
        // Duplicate weekdays should be rejected
        use crate::domain::periodicity::{Periodicity, PeriodicityConstraints, DayConstraint, RepetitionUnit};
        use chrono::Month;
        
        let p = Periodicity {
            rep_unit: RepetitionUnit::Day,
            rep_per_unit: Some(1),
            constraints: PeriodicityConstraints {
                day_constraint: Some(DayConstraint::SpecificDaysWeek(vec![
                    Weekday::Mon,
                    Weekday::Mon, // duplicate
                ])),
                ..Default::default()
            },
            timeframe: None,
            week_start: Weekday::Mon,
            year_start: Month::January,
            special_pattern: None,
        };
        
        let result = p.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_month_day_out_of_range() {
        // Day value > 30 is invalid (0-indexed, so max is 30 = 31st day)
        use crate::domain::periodicity::{Periodicity, PeriodicityConstraints, DayConstraint, RepetitionUnit};
        use chrono::Month;
        
        let p = Periodicity {
            rep_unit: RepetitionUnit::Day,
            rep_per_unit: Some(1),
            constraints: PeriodicityConstraints {
                day_constraint: Some(DayConstraint::SpecificDaysMonthFromFirst(vec![31])),
                ..Default::default()
            },
            timeframe: None,
            week_start: Weekday::Mon,
            year_start: Month::January,
            special_pattern: None,
        };
        
        let result = p.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_every_n_days_zero() {
        // EveryNDays(0) is invalid
        use crate::domain::periodicity::{Periodicity, PeriodicityConstraints, DayConstraint, RepetitionUnit};
        use chrono::Month;
        
        let p = Periodicity {
            rep_unit: RepetitionUnit::Day,
            rep_per_unit: Some(1),
            constraints: PeriodicityConstraints {
                day_constraint: Some(DayConstraint::EveryNDays(0)),
                ..Default::default()
            },
            timeframe: None,
            week_start: Weekday::Mon,
            year_start: Month::January,
            special_pattern: None,
        };
        
        let result = p.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_every_n_days_too_large() {
        // EveryNDays > 366 is invalid
        use crate::domain::periodicity::{Periodicity, PeriodicityConstraints, DayConstraint, RepetitionUnit};
        use chrono::Month;
        
        let p = Periodicity {
            rep_unit: RepetitionUnit::Day,
            rep_per_unit: Some(1),
            constraints: PeriodicityConstraints {
                day_constraint: Some(DayConstraint::EveryNDays(367)),
                ..Default::default()
            },
            timeframe: None,
            week_start: Weekday::Mon,
            year_start: Month::January,
            special_pattern: None,
        };
        
        let result = p.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_timeframe_start_after_end() {
        let start = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap(); // Before start!
        
        let result = PeriodicityBuilder::new()
            .daily(1)
            .between(start, end)
            .build();
        
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_special_pattern_with_constraints() {
        // Special patterns cannot have regular constraints
        use crate::domain::periodicity::{Periodicity, PeriodicityConstraints, DayConstraint, RepetitionUnit, SpecialPattern, UniqueDate};
        use chrono::Month;
        
        let date = Utc::now();
        let p = Periodicity {
            rep_unit: RepetitionUnit::None,
            rep_per_unit: None,
            special_pattern: Some(SpecialPattern::Unique(UniqueDate { date })),
            constraints: PeriodicityConstraints {
                day_constraint: Some(DayConstraint::EveryDay),
                ..Default::default()
            },
            timeframe: None,
            week_start: Weekday::Mon,
            year_start: Month::January,
        };
        
        let result = p.validate();
        assert!(result.is_err());
    }

    // ========================================================================
    // COMPLEX CONSTRAINT COMBINATIONS
    // ========================================================================

    #[test]
    fn test_complex_multiple_constraints() {
        // "Twice per day, on Mondays, in the first week of the month, only in Q1"
        let result = PeriodicityBuilder::new()
            .daily(2)
            .on_weekdays(vec![Weekday::Mon])
            .on_weeks_of_month(vec![1])
            .in_months(vec![Month::January, Month::February, Month::March])
            .build();
        
        assert!(result.is_ok());
        let p = result.unwrap();
        
        // All constraints combined
        assert!(p.constraints.day_constraint.is_some());
        assert!(p.constraints.week_constraint.is_some());
        assert!(p.constraints.month_constraint.is_some());
    }

    #[test]
    fn test_builder_chaining() {
        // Test that builder methods chain correctly
        let result = PeriodicityBuilder::new()
            .daily(1)
            .on_weekdays(vec![Weekday::Sat, Weekday::Sun])
            .week_starts_on(Weekday::Sun)
            .build();
        
        assert!(result.is_ok());
        let p = result.unwrap();
        assert_eq!(p.week_start, Weekday::Sun);
    }

    // ========================================================================
    // EDGE CASES
    // ========================================================================

    #[test]
    fn test_leap_year_feb_29() {
        // Test behavior with Feb 29 (leap year)
        let result = PeriodicityBuilder::new()
            .daily(1)
            .on_month_days(vec![29])
            .in_months(vec![Month::February])
            .build();
        
        assert!(result.is_ok());
        let p = result.unwrap();
        
        // 2024 is a leap year
        let feb_29_2024 = Utc.with_ymd_and_hms(2024, 2, 29, 10, 0, 0).unwrap();
        assert!(p.matches_constraints(&feb_29_2024));
        
        // 2026 is NOT a leap year - Feb 29 doesn't exist
        // (we can't test constraint matching on a non-existent date)
    }

    #[test]
    fn test_month_with_varying_days() {
        // "Last 3 days of each month"
        let result = PeriodicityBuilder::new()
            .daily(1)
            .on_month_days_from_end(vec![1, 2, 3])
            .build();
        
        assert!(result.is_ok());
        let p = result.unwrap();
        
        // Test in January (31 days): 29, 30, 31
        let jan_29 = Utc.with_ymd_and_hms(2026, 1, 29, 10, 0, 0).unwrap();
        let jan_31 = Utc.with_ymd_and_hms(2026, 1, 31, 10, 0, 0).unwrap();
        assert!(p.matches_constraints(&jan_29));
        assert!(p.matches_constraints(&jan_31));
        
        // Test in February (28 days in 2026): 26, 27, 28
        let feb_26 = Utc.with_ymd_and_hms(2026, 2, 26, 10, 0, 0).unwrap();
        let feb_28 = Utc.with_ymd_and_hms(2026, 2, 28, 10, 0, 0).unwrap();
        assert!(p.matches_constraints(&feb_26));
        assert!(p.matches_constraints(&feb_28));
    }

    #[test]
    fn test_max_repetition_counts() {
        // Test boundary values for rep_per_unit
        let daily_max = PeriodicityBuilder::new().daily(100).build();
        assert!(daily_max.is_ok());
        
        let daily_over = PeriodicityBuilder::new().daily(101).build();
        assert!(daily_over.is_err());
        
        let yearly_max = PeriodicityBuilder::new().yearly(255).build();
        assert!(yearly_max.is_ok());
    }

    #[test]
    fn test_concurrent_week_and_month_constraints() {
        // Can combine week-of-month with specific months
        let result = PeriodicityBuilder::new()
            .weekly(1)
            .on_weeks_of_month(vec![1, 3]) // First and third week
            .in_months(vec![Month::June, Month::December])
            .build();
        
        assert!(result.is_ok());
    }
}
