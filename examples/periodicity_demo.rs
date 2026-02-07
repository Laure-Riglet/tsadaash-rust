/// Example usage demonstrations for the Periodicity system
/// This file shows how to use the new constraint-based periodicity model

use tsadaash::domain::{
    Periodicity, PeriodicityBuilder, 
    NthWeekdayOfMonth
};
use chrono::{Utc, Weekday, Month, TimeZone};

fn main() {
    println!("=== Periodicity System Examples ===\n");
    
    // Example 1: Simple daily task
    println!("1. Simple Daily Task:");
    let daily = Periodicity::daily().unwrap();
    println!("   Created: {:?}", daily.rep_unit);
    println!("   Occurrences per unit: {:?}\n", daily.rep_per_unit);
    
    // Example 2: The user's example - 13th and 24th of Jan & Feb
    println!("2. User's Example - 13th and 24th of January and February:");
    let user_example = PeriodicityBuilder::new()
        .daily(1)
        .on_month_days(vec![13, 24])
        .in_months(vec![Month::January, Month::February])
        .build()
        .unwrap();
    
    // Test some dates
    let jan_13 = Utc.with_ymd_and_hms(2026, 1, 13, 10, 0, 0).unwrap();
    let jan_24 = Utc.with_ymd_and_hms(2026, 1, 24, 10, 0, 0).unwrap();
    let mar_13 = Utc.with_ymd_and_hms(2026, 3, 13, 10, 0, 0).unwrap();
    
    println!("   Jan 13 matches: {}", user_example.matches_constraints(&jan_13));
    println!("   Jan 24 matches: {}", user_example.matches_constraints(&jan_24));
    println!("   Mar 13 matches: {} (wrong month)\n", user_example.matches_constraints(&mar_13));
    
    // Example 3: Weekdays only
    println!("3. Weekdays Only (Monday-Friday):");
    let weekdays = Periodicity::on_weekdays(vec![
        Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri
    ]).unwrap();
    
    let monday = Utc.with_ymd_and_hms(2026, 2, 2, 10, 0, 0).unwrap();
    let saturday = Utc.with_ymd_and_hms(2026, 2, 7, 10, 0, 0).unwrap();
    
    println!("   Monday matches: {}", weekdays.matches_constraints(&monday));
    println!("   Saturday matches: {}\n", weekdays.matches_constraints(&saturday));
    
    // Example 4: First Monday of each month
    println!("4. First Monday of Each Month:");
    let first_mondays = PeriodicityBuilder::new()
        .daily(1)
        .on_nth_weekdays(vec![NthWeekdayOfMonth::first(Weekday::Mon)])
        .build()
        .unwrap();
    
    let first_monday = Utc.with_ymd_and_hms(2026, 2, 2, 10, 0, 0).unwrap();
    let second_monday = Utc.with_ymd_and_hms(2026, 2, 9, 10, 0, 0).unwrap();
    
    println!("   Feb 2 (1st Mon) matches: {}", first_mondays.matches_constraints(&first_monday));
    println!("   Feb 9 (2nd Mon) matches: {}\n", first_mondays.matches_constraints(&second_monday));
    
    // Example 5: Last day of each month
    println!("5. Last Day of Each Month:");
    let last_days = PeriodicityBuilder::new()
        .daily(1)
        .on_month_days_from_end(vec![1])
        .build()
        .unwrap();
    
    let jan_31 = Utc.with_ymd_and_hms(2026, 1, 31, 10, 0, 0).unwrap();
    let jan_30 = Utc.with_ymd_and_hms(2026, 1, 30, 10, 0, 0).unwrap();
    let feb_28 = Utc.with_ymd_and_hms(2026, 2, 28, 10, 0, 0).unwrap();
    
    println!("   Jan 31 matches: {}", last_days.matches_constraints(&jan_31));
    println!("   Jan 30 matches: {}", last_days.matches_constraints(&jan_30));
    println!("   Feb 28 matches: {} (last day of Feb)\n", last_days.matches_constraints(&feb_28));
    
    // Example 6: Complex combination - 3 times per day on Mondays in Q1
    println!("6. Complex: 3x/day on Mondays in Q1:");
    let complex = PeriodicityBuilder::new()
        .daily(3)
        .on_weekdays(vec![Weekday::Mon])
        .in_months(vec![Month::January, Month::February, Month::March])
        .build()
        .unwrap();
    
    println!("   Repetitions per day: {:?}", complex.rep_per_unit);
    println!("   Has day constraint: {}", complex.constraints.day_constraint.is_some());
    println!("   Has month constraint: {}\n", complex.constraints.month_constraint.is_some());
    
    // Example 7: One-time task
    println!("7. One-Time Task (Christmas 2026):");
    let christmas = Utc.with_ymd_and_hms(2026, 12, 25, 0, 0, 0).unwrap();
    let unique = Periodicity::unique(christmas).unwrap();
    
    println!("   Repetition unit: {:?}", unique.rep_unit);
    println!("   Has special pattern: {}\n", unique.special_pattern.is_some());
    
    // Example 8: With timeframe
    println!("8. Daily Task with Timeframe (Feb 2026 only):");
    let start = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap();
    
    let timeframe_task = PeriodicityBuilder::new()
        .daily(1)
        .between(start, end)
        .build()
        .unwrap();
    
    let in_feb = Utc.with_ymd_and_hms(2026, 2, 15, 10, 0, 0).unwrap();
    let in_march = Utc.with_ymd_and_hms(2026, 3, 15, 10, 0, 0).unwrap();
    
    println!("   Feb 15 in timeframe: {}", timeframe_task.is_within_timeframe(&in_feb));
    println!("   Mar 15 in timeframe: {}\n", timeframe_task.is_within_timeframe(&in_march));
    
    println!("=== All examples completed successfully! ===");
}
