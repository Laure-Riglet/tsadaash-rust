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
        NthWeekdayOfMonth, RepetitionUnit, ValidationError
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
            reference_date: None,
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
            reference_date: None,
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
            reference_date: None,
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
            reference_date: None,
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
            reference_date: None,
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
            reference_date: None,
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
            reference_date: None,
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
    
    // ========================================================================
    // WEEK-OF-MONTH CALCULATION TESTS
    // ========================================================================

    #[test]
    fn test_week_of_month_february_2026_monday_start() {
        // February 2026: starts Sunday Feb 1, ends Saturday Feb 28
        // With Monday week_start, weeks should be:
        // - Feb 1 (Sun): belongs to January (invalid = 255)
        // - Feb 2-8 (Mon-Sun): Week 0    // - Feb 9-15 (Mon-Sun): Week 1
        // - Feb 16-22 (Mon-Sun): Week 2
        // - Feb 23-Mar 1 (Mon-Sun): Week 3 (overflow to next month)
        
        // Feb 1 (Sunday) - should be invalid (belongs to previous month)
        let feb_1 = Utc.with_ymd_and_hms(2026, 2, 1, 12, 0, 0).unwrap();
        let week = Periodicity::week_of_month_from_first(&feb_1, Weekday::Mon);
        assert_eq!(week, 255, "Feb 1 (Sun) should be invalid (255)");
        
        // Feb 2 (Monday) - Week 0
        let feb_2 = Utc.with_ymd_and_hms(2026, 2, 2, 12, 0, 0).unwrap();
        assert_eq!(Periodicity::week_of_month_from_first(&feb_2, Weekday::Mon), 0);
        
        // Feb 8 (Sunday) - Still Week 0
        let feb_8 = Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap();
        assert_eq!(Periodicity::week_of_month_from_first(&feb_8, Weekday::Mon), 0);
        
        // Feb 9 (Monday) - Week 1
        let feb_9 = Utc.with_ymd_and_hms(2026, 2, 9, 12, 0, 0).unwrap();
        assert_eq!(Periodicity::week_of_month_from_first(&feb_9, Weekday::Mon), 1);
        
        // Feb 15 (Sunday) - Still Week 1
        let feb_15 = Utc.with_ymd_and_hms(2026, 2, 15, 12, 0, 0).unwrap();
        assert_eq!(Periodicity::week_of_month_from_first(&feb_15, Weekday::Mon), 1);
        
        // Feb 16 (Monday) - Week 2
        let feb_16 = Utc.with_ymd_and_hms(2026, 2, 16, 12, 0, 0).unwrap();
        assert_eq!(Periodicity::week_of_month_from_first(&feb_16, Weekday::Mon), 2);
        
        // Feb 22 (Sunday) - Still Week 2
        let feb_22 = Utc.with_ymd_and_hms(2026, 2, 22, 12, 0, 0).unwrap();
        assert_eq!(Periodicity::week_of_month_from_first(&feb_22, Weekday::Mon), 2);
        
        // Feb 23 (Monday) - Week 3
        let feb_23 = Utc.with_ymd_and_hms(2026, 2, 23, 12, 0, 0).unwrap();
        assert_eq!(Periodicity::week_of_month_from_first(&feb_23, Weekday::Mon), 3);
        
        // Feb 28 (Saturday) - Still Week 3
        let feb_28 = Utc.with_ymd_and_hms(2026, 2, 28, 12, 0, 0).unwrap();
        assert_eq!(Periodicity::week_of_month_from_first(&feb_28, Weekday::Mon), 3);
        
        // Verify February 2026 has 4 complete weeks with Monday start
        let weeks_count = Periodicity::weeks_in_month(2026, 2, Weekday::Mon);
        assert_eq!(weeks_count, 4, "February 2026 should have 4 weeks with Monday start");
    }

    #[test]
    fn test_week_constraint_first_two_weeks() {
        // Create periodicity: first 2 weeks of month (weeks 0 and 1 internally, but 1 and 2 for humans)
        let periodicity = PeriodicityBuilder::new()
            .daily(1)
            .on_weeks_of_month(vec![1, 2]) // 1-indexed: week 1 and week 2
            .week_starts_on(Weekday::Mon)
            .build()
            .unwrap();
        
        // February 2026 tests
        let feb_1 = Utc.with_ymd_and_hms(2026, 2, 1, 12, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&feb_1), "Feb 1 (Sun, pre-week-0) should not match");
        
        let feb_2 = Utc.with_ymd_and_hms(2026, 2, 2, 12, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&feb_2), "Feb 2 (Mon, week 0) should match");
        
        let feb_8 = Utc.with_ymd_and_hms(2026, 2, 8, 12, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&feb_8), "Feb 8 (Sun, week 0) should match");
        
        let feb_9 = Utc.with_ymd_and_hms(2026, 2, 9, 12, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&feb_9), "Feb 9 (Mon, week 1) should match");
        
        let feb_15 = Utc.with_ymd_and_hms(2026, 2, 15, 12, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&feb_15), "Feb 15 (Sun, week 1) should match");
        
        let feb_16 = Utc.with_ymd_and_hms(2026, 2, 16, 12, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&feb_16), "Feb 16 (Mon, week 2) should NOT match");
        
        let feb_23 = Utc.with_ymd_and_hms(2026, 2, 23, 12, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&feb_23), "Feb 23 (Mon, week 3) should NOT match");
    }

    #[test]
    fn test_weeks_in_different_months() {
        // January 2026: starts Thursday, ends Saturday (31 days)
        // With Monday start: Mon Jan 5 starts week 0
        let jan_weeks = Periodicity::weeks_in_month(2026, 1, Weekday::Mon);
        assert!(jan_weeks >= 4 && jan_weeks <= 5, "January 2026 should have 4-5 weeks, got {}", jan_weeks);
        
        // March 2026: starts Sunday, ends Tuesday (31 days)  
        // With Monday start: Mon Mar 2 starts week 0
        let mar_weeks = Periodicity::weeks_in_month(2026, 3, Weekday::Mon);
        assert!(mar_weeks >= 4 && mar_weeks <= 5, "March 2026 should have 4-5 weeks, got {}", mar_weeks);
        
        // February 2026: 28 days, starts Sunday
        let feb_weeks = Periodicity::weeks_in_month(2026, 2, Weekday::Mon);
        assert_eq!(feb_weeks, 4, "February 2026 should have exactly 4 weeks");
    }

    // ========================================================================
    // EVERY N* ROLLING PATTERN TESTS
    // ========================================================================

    #[test]
    fn test_every_n_days_with_reference_date() {
        // EveryNDays(3) with explicit reference date
        let reference = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        
        let periodicity = PeriodicityBuilder::new()
            .daily(1)
            .every_n_days(3)
            .with_reference_date(reference)
            .build()
            .unwrap();
        
        // Day 0 (reference): Jan 1 - should match
        assert!(periodicity.matches_constraints(&reference), "Jan 1 (day 0) should match");
        
        // Day 3: Jan 4 - should match
        let jan_4 = Utc.with_ymd_and_hms(2026, 1, 4, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&jan_4), "Jan 4 (day 3) should match");
        
        // Day 2: Jan 3 - should NOT match
        let jan_3 = Utc.with_ymd_and_hms(2026, 1, 3, 0, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&jan_3), "Jan 3 (day 2) should NOT match");
        
        // Day 6: Jan 7 - should match
        let jan_7 = Utc.with_ymd_and_hms(2026, 1, 7, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&jan_7), "Jan 7 (day 6) should match");
        
        // Day 5: Jan 6 - should NOT match
        let jan_6 = Utc.with_ymd_and_hms(2026, 1, 6, 0, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&jan_6), "Jan 6 (day 5) should NOT match");
    }

    #[test]
    fn test_every_n_weeks_with_reference_date() {
        // EveryNWeeks(2) with Monday start, reference Jan 5 (Monday)
        let reference = Utc.with_ymd_and_hms(2026, 1, 5, 0, 0, 0).unwrap(); // Monday
        
        let periodicity = PeriodicityBuilder::new()
            .weekly(1)
            .every_n_weeks(2)
            .week_starts_on(Weekday::Mon)
            .with_reference_date(reference)
            .build()
            .unwrap();
        
        // Week 0: Jan 5-11 (Mon-Sun) - should match
        let jan_5 = Utc.with_ymd_and_hms(2026, 1, 5, 12, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&jan_5), "Jan 5 (Mon, week 0) should match");
        
        let jan_10 = Utc.with_ymd_and_hms(2026, 1, 10, 12, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&jan_10), "Jan 10 (Sat, week 0) should match");
        
        // Week 1: Jan 12-18 (Mon-Sun) - should NOT match
        let jan_12 = Utc.with_ymd_and_hms(2026, 1, 12, 12, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&jan_12), "Jan 12 (Mon, week 1) should NOT match");
        
        // Week 2: Jan 19-25 (Mon-Sun) - should match
        let jan_19 = Utc.with_ymd_and_hms(2026, 1, 19, 12, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&jan_19), "Jan 19 (Mon, week 2) should match");
        
        let jan_23 = Utc.with_ymd_and_hms(2026, 1, 23, 12, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&jan_23), "Jan 23 (Fri, week 2) should match");
        
        // Week 3: Jan 26-Feb 1 (Mon-Sun) - should NOT match
        let jan_26 = Utc.with_ymd_and_hms(2026, 1, 26, 12, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&jan_26), "Jan 26 (Mon, week 3) should NOT match");
    }

    #[test]
    fn test_every_n_months_with_reference_date() {
        // EveryNMonths(2) - every 2 months starting from January
        let reference = Utc.with_ymd_and_hms(2026, 1, 15, 0, 0, 0).unwrap();
        
        let periodicity = PeriodicityBuilder::new()
            .monthly(1)
            .every_n_months(2)
            .with_reference_date(reference)
            .build()
            .unwrap();
        
        // Month 0: January - should match
        let jan_20 = Utc.with_ymd_and_hms(2026, 1, 20, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&jan_20), "January (month 0) should match");
        
        // Month 1: February - should NOT match
        let feb_15 = Utc.with_ymd_and_hms(2026, 2, 15, 0, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&feb_15), "February (month 1) should NOT match");
        
        // Month 2: March - should match
        let mar_15 = Utc.with_ymd_and_hms(2026, 3, 15, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&mar_15), "March (month 2) should match");
        
        // Month 3: April - should NOT match
        let apr_15 = Utc.with_ymd_and_hms(2026, 4, 15, 0, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&apr_15), "April (month 3) should NOT match");
        
        // Month 4: May - should match
        let may_15 = Utc.with_ymd_and_hms(2026, 5, 15, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&may_15), "May (month 4) should match");
    }

    #[test]
    fn test_every_n_years_with_reference_date() {
        // EveryNYears(2) - every 2 years starting from 2026
        let reference = Utc.with_ymd_and_hms(2026, 6, 15, 0, 0, 0).unwrap();
        
        let periodicity = PeriodicityBuilder::new()
            .yearly(1)
            .every_n_years(2)
            .with_reference_date(reference)
            .build()
            .unwrap();
        
        // Year 0: 2026 - should match
        let y2026 = Utc.with_ymd_and_hms(2026, 8, 20, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&y2026), "2026 (year 0) should match");
        
        // Year 1: 2027 - should NOT match
        let y2027 = Utc.with_ymd_and_hms(2027, 6, 15, 0, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&y2027), "2027 (year 1) should NOT match");
        
        // Year 2: 2028 - should match
        let y2028 = Utc.with_ymd_and_hms(2028, 6, 15, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&y2028), "2028 (year 2) should match");
        
        // Year 4: 2030 - should match
        let y2030 = Utc.with_ymd_and_hms(2030, 6, 15, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&y2030), "2030 (year 4) should match");
    }

    #[test]
    fn test_reference_date_fallback_to_timeframe() {
        // When reference_date is not set, should use timeframe.start
        let timeframe_start = Utc.with_ymd_and_hms(2026, 1, 10, 0, 0, 0).unwrap();
        let timeframe_end = Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59).unwrap();
        
        let periodicity = PeriodicityBuilder::new()
            .daily(1)
            .every_n_days(5)
            .between(timeframe_start, timeframe_end)
            // Note: NOT setting reference_date explicitly
            .build()
            .unwrap();
        
        // Day 0: Jan 10 (timeframe start) - should match
        assert!(periodicity.matches_constraints(&timeframe_start), "Jan 10 (day 0) should match");
        
        // Day 5: Jan 15 - should match
        let jan_15 = Utc.with_ymd_and_hms(2026, 1, 15, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&jan_15), "Jan 15 (day 5) should match");
        
        // Day 4: Jan 14 - should NOT match
        let jan_14 = Utc.with_ymd_and_hms(2026, 1, 14, 0, 0, 0).unwrap();
        assert!(!periodicity.matches_constraints(&jan_14), "Jan 14 (day 4) should NOT match");
    }

    #[test]
    fn test_reference_date_fallback_to_current_date() {
        // When neither reference_date nor timeframe is set, uses current date being checked
        let periodicity = PeriodicityBuilder::new()
            .daily(1)
            .every_n_days(7)
            // Note: NOT setting reference_date or timeframe
            .build()
            .unwrap();
        
        // Any date will match because it's 0 days from itself (itself is the fallback reference)
        let any_date = Utc.with_ymd_and_hms(2026, 5, 20, 0, 0, 0).unwrap();
        assert!(periodicity.matches_constraints(&any_date), "Any date should match (0 days from itself)");
    }
}
