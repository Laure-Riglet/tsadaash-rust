# Periodicity Quick Reference

## Construction

### Simple Patterns

```rust
Periodicity::daily()?                    // Once per day, every day
Periodicity::weekly()?                   // Once per week, every week
Periodicity::monthly()?                  // Once per month, every month
Periodicity::yearly()?                   // Once per year, every year
Periodicity::unique(date)?               // One-time task
Periodicity::on_weekdays(vec![Mon, Fri])? // Specific days of week
```

### Builder Pattern

```rust
PeriodicityBuilder::new()
    .daily(3)                // 3 times per day
    .on_weekdays(vec![Mon])  // Only on Mondays
    .in_months(vec![Jan])    // Only in January
    .build()?
```

## Core Concepts

**Repetition:** HOW OFTEN (frequency)

- `rep_unit`: Day, Week, Month, Year, None
- `rep_per_unit`: Count (1-255)

**Constraints:** WHEN (filters, AND logic)

- `day_constraint`: Filter specific days
- `week_constraint`: Filter specific weeks
- `month_constraint`: Filter specific months
- `year_constraint`: Filter specific years

## Constraint Types

### Day

```rust
.every_day()                          // No filter
.every_n_days(3)                      // Every 3 days
.on_weekdays(vec![Mon, Wed, Fri])     // Specific weekdays
.on_month_days(vec![1, 15])           // 1st and 15th of month
.on_month_days_from_end(vec![1])      // Last day of month
.on_nth_weekdays(vec![               // First Monday, last Friday
    NthWeekdayOfMonth::first(Mon),
    NthWeekdayOfMonth::last(Fri)
])
```

### Week

```rust
.every_week()                         // No filter
.every_n_weeks(2)                     // Every 2 weeks
.on_weeks_of_month(vec![1, 3])        // 1st and 3rd week
.on_weeks_of_month_from_end(vec![1])  // Last week
```

### Month

```rust
.every_month()                        // No filter
.every_n_months(3)                    // Quarterly
.in_months(vec![Jan, Feb, Mar])       // Q1 only
```

### Year

```rust
.every_year()                         // No filter
.every_n_years(2)                     // Biennial
.in_years(vec![2025, 2026])           // Specific years
```

### Timeframe

```rust
.between(start, end)                  // Valid period
.starting_from(start)                 // Open-ended start
.until(end)                           // Open-ended end
```

### Calendar

```rust
.week_starts_on(Sunday)               // Custom week start
.year_starts_in(April)                // Fiscal year
```

## Common Patterns

### Weekdays Only

```rust
Periodicity::on_weekdays(vec![Mon, Tue, Wed, Thu, Fri])?
```

### Weekends Only

```rust
Periodicity::on_weekdays(vec![Sat, Sun])?
```

### First Day of Month

```rust
PeriodicityBuilder::new()
    .daily(1)
    .on_month_days(vec![1])
    .build()?
```

### Last Day of Month

```rust
PeriodicityBuilder::new()
    .daily(1)
    .on_month_days_from_end(vec![1])
    .build()?
```

### Quarterly (First Day)

```rust
PeriodicityBuilder::new()
    .daily(1)
    .on_month_days(vec![1])
    .in_months(vec![Jan, Apr, Jul, Oct])
    .build()?
```

### Biweekly (Every 2 Weeks)

```rust
PeriodicityBuilder::new()
    .weekly(1)
    .every_n_weeks(2)
    .build()?
```

### First Monday of Month

```rust
PeriodicityBuilder::new()
    .daily(1)
    .on_nth_weekdays(vec![NthWeekdayOfMonth::first(Mon)])
    .build()?
```

### Last Friday of Month

```rust
PeriodicityBuilder::new()
    .daily(1)
    .on_nth_weekdays(vec![NthWeekdayOfMonth::last(Fri)])
    .build()?
```

### Multiple Times Per Day

```rust
PeriodicityBuilder::new()
    .daily(3)  // Morning, afternoon, evening
    .every_day()
    .build()?
```

### Complex: Twice Daily on Weekdays in Q1

```rust
PeriodicityBuilder::new()
    .daily(2)
    .on_weekdays(vec![Mon, Tue, Wed, Thu, Fri])
    .in_months(vec![Jan, Feb, Mar])
    .build()?
```

## NthWeekdayOfMonth Helpers

```rust
NthWeekdayOfMonth::first(Mon)        // 1st Monday
NthWeekdayOfMonth::second(Tue)       // 2nd Tuesday
NthWeekdayOfMonth::third(Wed)        // 3rd Wednesday
NthWeekdayOfMonth::fourth(Thu)       // 4th Thursday
NthWeekdayOfMonth::last(Fri)         // Last Friday
NthWeekdayOfMonth::second_last(Sat)  // 2nd-to-last Saturday
```

## Usage

### Check If Date Matches

```rust
let date = Utc.with_ymd_and_hms(2026, 1, 13, 10, 0, 0)?;

if periodicity.matches_constraints(&date) {
    println!("Task should occur on this date");
}
```

### Check Timeframe

```rust
if periodicity.is_within_timeframe(&date) {
    println!("Date is within valid period");
}
```

### Combined Check

```rust
if periodicity.matches_constraints(&date) &&
   periodicity.is_within_timeframe(&date) {
    println!("Task is valid for this date");
}
```

### Validate Configuration

```rust
match periodicity.validate() {
    Ok(()) => println!("Valid configuration"),
    Err(e) => eprintln!("Validation error: {}", e),
}
```

## Error Handling

```rust
use crate::domain::ValidationError;

match PeriodicityBuilder::new().daily(1).build() {
    Ok(p) => { /* use periodicity */ },
    Err(ValidationError::MissingRequired { field, reason }) => {
        eprintln!("Missing {}: {}", field, reason);
    },
    Err(ValidationError::InvalidValue { field, value, reason }) => {
        eprintln!("Invalid {} value '{}': {}", field, value, reason);
    },
    Err(e) => {
        eprintln!("Validation error: {}", e);
    }
}
```

## Value Ranges

| Type       | Field                | Min  | Max  | Notes            |
| ---------- | -------------------- | ---- | ---- | ---------------- |
| Day        | EveryNDays           | 1    | 366  |                  |
| Day        | MonthDays            | 0    | 30   | 0-indexed        |
| Week       | EveryNWeeks          | 1    | 52   |                  |
| Week       | WeeksOfMonth         | 0    | 4    | 0-indexed        |
| Month      | EveryNMonths         | 1    | 12   |                  |
| Year       | EveryNYears          | 1    | 100  |                  |
| Year       | SpecificYears        | 1900 | 2200 |                  |
| Repetition | rep_per_unit (Day)   | 1    | 100  | Practical limit  |
| Repetition | rep_per_unit (Week)  | 1    | 50   | Practical limit  |
| Repetition | rep_per_unit (Month) | 1    | 100  | Practical limit  |
| Repetition | rep_per_unit (Year)  | 1    | 366  | Max once per day |

## Indexing Notes

**0-Indexed (Internal):**

- Month days: 0 = 1st, 30 = 31st
- Weeks of month: 0 = first, 4 = fifth
- Week positions: 0 = first, 4 = fifth

**1-Indexed (Builder Auto-Converts):**

- `.on_month_days(vec![1, 15])` → Internal [0, 14]
- `.on_weeks_of_month(vec![1, 3])` → Internal [0, 2]

**Human-Friendly (Month enum):**

- `Month::January` = 1
- `Month::December` = 12

## Import Statement

```rust
use crate::domain::{
    Periodicity,
    PeriodicityBuilder,
    DayConstraint,
    WeekConstraint,
    MonthConstraint,
    YearConstraint,
    NthWeekdayOfMonth,
    ValidationError,
};
use chrono::{Weekday, Month, Utc, TimeZone};
```

## Tips

1. **Always use builder** for construction
2. **Handle `Result`** - validation can fail
3. **0-indexed internally** but builder accepts 1-indexed
4. **Constraints are AND** - all must match
5. **Test edge cases** - month boundaries, leap years
6. **Validate before persisting**
7. **Document complex patterns** in task descriptions
