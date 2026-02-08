//! # Schedule Module
//! 
//! Domain module for managing user schedule templates and time-based availability.
//! 
//! ## Features
//! 
//! - **Weekly Schedule Templates**: Define recurring availability patterns
//! - **Time Block Expansion**: Convert templates into concrete time blocks for date ranges
//! - **Task Matching**: Determine if tasks can be scheduled in specific time blocks
//! - **Capability Modeling**: Track hands, eyes, speech, cognitive, device, and mobility
//! - **Location Constraints**: Require or restrict tasks based on user location
//! - **Priority-Based Conflict Resolution**: Handle overlapping rules intelligently
//! - **Overnight Support**: Handle rules that span midnight
//! 
//! ## Example Usage
//! 
//! ```rust
//! use tsadaash::domain::entities::schedule::{
//!     types::{AvailabilityKind, CapabilitySet, LocationConstraint},
//!     template::{RecurringRule, ScheduleTemplate},
//!     expansion::expand_template,
//! };
//! use chrono::{Weekday, NaiveTime, FixedOffset, TimeZone};
//! 
//! // Create a work hours rule (Mon-Fri 9-5)
//! let work_rule = RecurringRule::new(
//!     vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri],
//!     NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
//!     NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
//!     AvailabilityKind::BusyButFlexible,
//!     CapabilitySet::free(),
//!     LocationConstraint::Any,
//!     Some("Work".to_string()),
//!     0,
//! ).unwrap();
//! 
//! // Create a schedule template
//! let template = ScheduleTemplate::new(
//!     1,
//!     1,
//!     "My Schedule".to_string(),
//!     "America/New_York".to_string(),
//!     vec![work_rule],
//! ).unwrap();
//! 
//! // Expand for a date range
//! let tz = FixedOffset::west_opt(5 * 3600).unwrap();
//! let start = tz.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap();
//! let end = tz.with_ymd_and_hms(2026, 2, 17, 0, 0, 0).unwrap();
//! 
//! let blocks = expand_template(&template, start, end);
//! // blocks now contains concrete time blocks for the week
//! ```

/// Core types: availability, capabilities, location constraints, constants
pub mod types;

/// Template types: RecurringRule and ScheduleTemplate
pub mod template;

/// Expansion engine: convert templates to concrete time blocks
pub mod expansion;

/// Task matching: determine if tasks fit in time blocks
pub mod matching;

// Integration tests
#[cfg(test)]
mod tests;

// ========================================================================
// PUBLIC API RE-EXPORTS
// ========================================================================

// Core types
pub use types::{
    AvailabilityKind,
    AvailabilityLevel,
    CapabilitySet,
    DeviceAccess,
    LocationConstraint,
    Mobility,
    UnavailableReason,
    busy_flex_max_device,
    busy_flex_max_eyes,
    busy_flex_max_hands,
    busy_flex_max_minutes,
};

// Template types
pub use template::{RecurringRule, ScheduleTemplate};

// Expansion
pub use expansion::{expand_template, TimeBlock};

// Matching
pub use matching::{can_schedule_task_in_block, find_candidate_slots, SchedulableTask};
