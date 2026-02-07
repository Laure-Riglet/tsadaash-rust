use chrono::{Month, NaiveTime, Weekday};
use crate::domain::Timezone;

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password: String,

    // ── TIMEZONE SETTINGS ───────────────────────────────────
    /// User's timezone (e.g., "America/New_York", "Europe/London")
    pub timezone: Timezone,

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
    /// Creates a new user with the given timezone
    /// 
    /// # Errors
    /// Returns `TimezoneError` if the timezone is invalid
    pub fn new(
        username: String,
        email: String,
        password: String,
        timezone: Timezone,
    ) -> Self {
        Self {
            username,
            email,
            password,
            timezone,
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
        timezone: Timezone,
        week_start: Weekday,
        year_start: Month,
        day_start: NaiveTime,
    ) -> Self {
        Self {
            username,
            email,
            password,
            timezone,
            week_start,
            year_start,
            day_start,
        }
    }
    
    // ── TIMEZONE SETTER ─────────────────────────────────────
    
    /// Updates the user's timezone
    pub fn set_timezone(&mut self, timezone: Timezone) {
        self.timezone = timezone;
    }
    
    // ── CALENDAR SETTINGS SETTERS ──────────────────────────
    
    /// Sets the first day of the week
    pub fn set_week_start(&mut self, weekday: Weekday) {
        self.week_start = weekday;
    }
    
    /// Sets the first month of the year (for fiscal year support)
    pub fn set_year_start(&mut self, month: Month) {
        self.year_start = month;
    }
    
    /// Sets the time of day when a new day begins
    /// 
    /// # Example
    /// ```
    /// # use tsadaash::domain::{User, Timezone, Continents};
    /// # use chrono::NaiveTime;
    /// let timezone = Timezone::new(Continents::America, "New_York".to_string()).unwrap();
    /// let mut user = User::new(
    ///     "user".to_string(),
    ///     "user@example.com".to_string(),
    ///     "password".to_string(),
    ///     timezone,
    /// );
    /// 
    /// // Night shift worker: day starts at 6 PM
    /// user.set_day_start(NaiveTime::from_hms_opt(18, 0, 0).unwrap());
    /// ```
    pub fn set_day_start(&mut self, time: NaiveTime) {
        self.day_start = time;
    }
}