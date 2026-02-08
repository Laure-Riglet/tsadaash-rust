/// Clock abstraction for time injection

use chrono::{DateTime, Utc};

/// Trait for providing current time
/// This allows for easy testing and deterministic behavior
pub trait Clock {
    /// Get the current UTC time
    fn now(&self) -> DateTime<Utc>;
}

/// System clock implementation using actual system time
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Fixed clock for testing (always returns the same time)
#[cfg(test)]
pub struct FixedClock {
    time: DateTime<Utc>,
}

#[cfg(test)]
impl FixedClock {
    pub fn new(time: DateTime<Utc>) -> Self {
        Self { time }
    }
}

#[cfg(test)]
impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.time
    }
}
