use chrono::{NaiveTime, Weekday};
use super::types::{AvailabilityKind, CapabilitySet, LocationConstraint};

// ========================================================================
// RECURRING RULE
// ========================================================================

/// Represents a recurring time block in a weekly schedule template
/// 
/// # Overnight Rules
/// If `end <= start`, the rule spans midnight into the next day.
/// For example, a rule with start=23:00 and end=07:00 runs from 11 PM
/// through midnight into 7 AM the next day.
#[derive(Debug, Clone, PartialEq)]
pub struct RecurringRule {
    /// Days of the week this rule applies to
    pub days: Vec<Weekday>,
    
    /// Start time (local time-of-day)
    pub start: NaiveTime,
    
    /// End time (local time-of-day, can be <= start for overnight rules)
    pub end: NaiveTime,
    
    /// Availability status during this period
    pub availability: AvailabilityKind,
    
    /// Capabilities available during this period
    pub capabilities: CapabilitySet,
    
    /// Location constraint for this period
    pub location_constraint: LocationConstraint,
    
    /// Optional label for display/debugging
    pub label: Option<String>,
    
    /// Priority for conflict resolution (higher wins)
    pub priority: i16,
}

impl RecurringRule {
    /// Check if this rule represents an overnight period
    pub fn is_overnight(&self) -> bool {
        self.end <= self.start
    }

    /// Create a new recurring rule with validation
    pub fn new(
        days: Vec<Weekday>,
        start: NaiveTime,
        end: NaiveTime,
        availability: AvailabilityKind,
        capabilities: CapabilitySet,
        location_constraint: LocationConstraint,
        label: Option<String>,
        priority: i16,
    ) -> Result<Self, String> {
        if days.is_empty() {
            return Err("RecurringRule must have at least one day".to_string());
        }

        Ok(Self {
            days,
            start,
            end,
            availability,
            capabilities,
            location_constraint,
            label,
            priority,
        })
    }
}

// ========================================================================
// SCHEDULE TEMPLATE
// ========================================================================

/// A weekly schedule template for a user
/// 
/// Contains a set of recurring rules that define availability patterns
/// throughout the week. Rules can overlap and are resolved by priority.
/// 
/// # Design Note
/// This entity does not contain persistence IDs (id, user_id).
/// Those are infrastructure concerns managed by repositories.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleTemplate {
    pub name: String,
    
    /// IANA timezone name (e.g., "America/New_York")
    pub timezone: String,
    
    /// Recurring rules that define the schedule
    pub rules: Vec<RecurringRule>,
}

impl ScheduleTemplate {
    /// Create a new schedule template with validation
    pub fn new(
        name: String,
        timezone: String,
        rules: Vec<RecurringRule>,
    ) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Schedule template name cannot be empty".to_string());
        }

        if timezone.trim().is_empty() {
            return Err("Timezone cannot be empty".to_string());
        }

        Ok(Self {
            name: name.trim().to_string(),
            timezone,
            rules,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::schedule::types::{AvailabilityKind, CapabilitySet, LocationConstraint};

    #[test]
    fn test_recurring_rule_is_overnight() {
        let rule_normal = RecurringRule::new(
            vec![Weekday::Mon],
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            None,
            0,
        ).unwrap();
        assert!(!rule_normal.is_overnight());

        let rule_overnight = RecurringRule::new(
            vec![Weekday::Mon],
            NaiveTime::from_hms_opt(23, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            None,
            0,
        ).unwrap();
        assert!(rule_overnight.is_overnight());
    }

    #[test]
    fn test_recurring_rule_validation() {
        // Empty days should fail
        let result = RecurringRule::new(
            vec![],
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            None,
            0,
        );
        assert!(result.is_err());

        // Valid rule should succeed
        let result = RecurringRule::new(
            vec![Weekday::Mon, Weekday::Tue],
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Work".to_string()),
            5,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_schedule_template_validation() {
        // Empty name should fail
        let result = ScheduleTemplate::new(
            "".to_string(),
            "America/New_York".to_string(),
            vec![],
        );
        assert!(result.is_err());

        // Empty timezone should fail
        let result = ScheduleTemplate::new(
            "My Schedule".to_string(),
            "".to_string(),
            vec![],
        );
        assert!(result.is_err());

        // Valid template should succeed
        let result = ScheduleTemplate::new(
            "My Schedule".to_string(),
            "America/New_York".to_string(),
            vec![],
        );
        assert!(result.is_ok());
    }
}
