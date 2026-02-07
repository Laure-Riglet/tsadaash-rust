use chrono::{Month, NaiveTime, Weekday};

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password: String,

    // ── TIMEZONE SETTINGS ───────────────────────────────────
    /// Timezone continent (e.g., "America", "Europe")
    pub tz_continent: String,
    /// Timezone city (e.g., "New_York", "London")
    pub tz_city: String,

    // ── CALENDAR SETTINGS ────────────────────────────────────
    
    /// First day of the week (for week-based calculations)
    /// Default: Monday
    pub week_start: Weekday,
    
    /// First month of the year (for year-based calculations)
    /// Default: January (for fiscal years, could be different)
    pub year_start: Month,
    
    /// Time of day when a new day begins (for daily task boundaries)
    /// Default: 00:00:00 (midnight)
    /// 
    /// # Use Case
    /// If set to 05:00:00, then "February 7th" runs from Feb 7 05:00:00 to Feb 8 04:59:59.
    /// Useful for users who consider their "day" to start at a different time
    /// (e.g., night shift workers, or "today ends when I go to sleep at 5 AM").
    pub day_start: NaiveTime,
}

impl User {
    pub fn new(
        username: String,
        email: String,
        password: String,
        tz_continent: String,
        tz_city: String,
    ) -> Self {
        Self {
            username,
            email,
            password,
            tz_continent,
            tz_city,
            week_start: Weekday::Mon,
            year_start: Month::January,
            day_start: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        }
    }
    
    /// Create a user with custom calendar settings
    pub fn with_calendar_settings(
        username: String,
        email: String,
        password: String,
        tz_continent: String,
        tz_city: String,
        week_start: Weekday,
        year_start: Month,
        day_start: NaiveTime,
    ) -> Self {
        Self {
            username,
            email,
            password,
            tz_continent,
            tz_city,
            week_start,
            year_start,
            day_start,
        }
    }
}