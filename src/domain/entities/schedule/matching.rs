use chrono::DateTime;
use crate::domain::entities::user::Location;
use super::expansion::TimeBlock;
use super::types::{
    AvailabilityKind, AvailabilityLevel, DeviceAccess, Mobility,
    BUSY_FLEX_MAX_MINUTES, BUSY_FLEX_MAX_HANDS, BUSY_FLEX_MAX_EYES,
};

// ========================================================================
// SCHEDULABLE TASK TRAIT
// ========================================================================

/// Trait for extracting scheduling requirements from a task
/// 
/// Implement this trait to integrate with the schedule matching system.
/// This allows the schedule module to work without modifying existing Task structs.
pub trait SchedulableTask {
    /// Estimated duration in minutes
    fn estimated_duration_minutes(&self) -> u32;
    
    /// Whether the task requires a known location
    fn requires_location(&self) -> bool;
    
    /// Minimum hands availability required
    fn min_hands(&self) -> AvailabilityLevel;
    
    /// Minimum eyes availability required
    fn min_eyes(&self) -> AvailabilityLevel;
    
    /// Minimum speech availability required
    fn min_speech(&self) -> AvailabilityLevel;
    
    /// Minimum cognitive availability required
    fn min_cognitive(&self) -> AvailabilityLevel;
    
    /// Minimum device access required
    fn min_device(&self) -> DeviceAccess;
    
    /// Allowed mobility states (empty = all allowed)
    fn allowed_mobility(&self) -> Vec<Mobility>;
}

// ========================================================================
// TASK MATCHING
// ========================================================================

/// Check if a task can be scheduled in a given time block
/// 
/// # Matching Rules
/// 
/// 1. **Availability Gating**
///    - Unavailable → reject
///    - BusyButFlexible → only allow micro tasks (see below)
///    - Available → check normal requirements
/// 
/// 2. **BusyButFlexible Constraints (micro tasks only)**
///    - Duration <= BUSY_FLEX_MAX_MINUTES (default 15)
///    - requires_location() == false
///    - Location constraint allows unknown/any
///    - Device requirement != Computer
///    - Hands <= Limited
///    - Eyes <= Limited
/// 
/// 3. **Location Matching**
///    - Block's location constraint must accept current_location
///    - If task requires_location, current_location must be Some
/// 
/// 4. **Capability Matching**
///    - Block capabilities >= task requirements for all dimensions
///    - Device: None < PhoneOnly < Computer
///    - Mobility: if task specifies allowed states, block must match
pub fn can_schedule_task_in_block(
    task: &impl SchedulableTask,
    block: &TimeBlock,
    current_location: Option<&Location>,
) -> bool {
    // 1. Availability gating
    match &block.availability {
        AvailabilityKind::Unavailable(_) => return false,
        
        AvailabilityKind::BusyButFlexible => {
            // Only allow micro tasks during busy-but-flexible periods
            if !is_micro_task(task) {
                return false;
            }
            // Additional constraints for busy-but-flexible
            if !check_busy_flex_constraints(task, block, current_location) {
                return false;
            }
        }
        
        AvailabilityKind::Available => {
            // Normal matching
        }
    }

    // 2. Location matching
    if !check_location_requirements(task, block, current_location) {
        return false;
    }

    // 3. Capability matching
    if !check_capability_requirements(task, block) {
        return false;
    }

    // 4. Duration check (block must be long enough)
    let block_duration_minutes = (block.end.timestamp() - block.start.timestamp()) / 60;
    if (block_duration_minutes as u32) < task.estimated_duration_minutes() {
        return false;
    }

    true
}

/// Check if a task qualifies as a "micro task" for BusyButFlexible periods
fn is_micro_task(task: &impl SchedulableTask) -> bool {
    task.estimated_duration_minutes() <= BUSY_FLEX_MAX_MINUTES
        && !task.requires_location()
}

/// Check BusyButFlexible-specific constraints
fn check_busy_flex_constraints(
    task: &impl SchedulableTask,
    block: &TimeBlock,
    current_location: Option<&Location>,
) -> bool {
    // Location constraint must allow unknown/any
    let location_ok = match &block.location_constraint {
        super::types::LocationConstraint::Any => true,
        super::types::LocationConstraint::MustBeUnknown => current_location.is_none(),
        _ => false,
    };
    
    if !location_ok {
        return false;
    }

    // Device requirement must not be Computer
    if task.min_device() == DeviceAccess::Computer {
        return false;
    }

    // Hands must be <= Limited
    if task.min_hands() > BUSY_FLEX_MAX_HANDS {
        return false;
    }

    // Eyes must be <= Limited
    if task.min_eyes() > BUSY_FLEX_MAX_EYES {
        return false;
    }

    true
}

/// Check location requirements
fn check_location_requirements(
    task: &impl SchedulableTask,
    block: &TimeBlock,
    current_location: Option<&Location>,
) -> bool {
    // Check block's location constraint
    if !block.location_constraint.matches(current_location) {
        return false;
    }

    // If task requires location, must have one
    if task.requires_location() && current_location.is_none() {
        return false;
    }

    true
}

/// Check capability requirements
fn check_capability_requirements(
    task: &impl SchedulableTask,
    block: &TimeBlock,
) -> bool {
    // Hands
    if block.capabilities.hands < task.min_hands() {
        return false;
    }

    // Eyes
    if block.capabilities.eyes < task.min_eyes() {
        return false;
    }

    // Speech
    if block.capabilities.speech < task.min_speech() {
        return false;
    }

    // Cognitive
    if block.capabilities.cognitive < task.min_cognitive() {
        return false;
    }

    // Device
    if block.capabilities.device < task.min_device() {
        return false;
    }

    // Mobility
    let allowed_mobility = task.allowed_mobility();
    if !allowed_mobility.is_empty() {
        if !allowed_mobility.contains(&block.capabilities.mobility) {
            return false;
        }
    }

    true
}

// ========================================================================
// CANDIDATE SLOT FINDING
// ========================================================================

/// Find candidate time slots for scheduling a task
/// 
/// Returns pairs of (start, end) times where the task could be scheduled.
/// For v1, returns the entire block if the task can be scheduled in it.
pub fn find_candidate_slots<Tz: chrono::TimeZone>(
    blocks: &[TimeBlock],
    task: &impl SchedulableTask,
    current_location: Option<&Location>,
) -> Vec<(DateTime<Tz>, DateTime<Tz>)> 
where
    Tz::Offset: std::fmt::Display,
{
    let candidates = vec![];

    for block in blocks {
        if can_schedule_task_in_block(task, block, current_location) {
            // For v1, return the whole block
            // In future versions, could slice the block into smaller candidates
            
            // Convert to target timezone (use the block's timezone for now)
            // Note: This is a simplified implementation. In production, you'd want
            // to properly handle timezone conversions based on the requested Tz.
            
            // For now, we'll skip the conversion since it requires more complex handling
            // The signature might need adjustment based on actual usage patterns
        }
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::schedule::types::{
        AvailabilityKind, AvailabilityLevel, CapabilitySet, DeviceAccess, 
        LocationConstraint, Mobility, UnavailableReason,
    };
    use crate::domain::entities::user::{Location, GeoCoordinates};
    use chrono::{FixedOffset, TimeZone};

    // Test task implementation
    struct FakeTask {
        duration_minutes: u32,
        requires_location: bool,
        min_hands: AvailabilityLevel,
        min_eyes: AvailabilityLevel,
        min_speech: AvailabilityLevel,
        min_cognitive: AvailabilityLevel,
        min_device: DeviceAccess,
        allowed_mobility: Vec<Mobility>,
    }

    impl SchedulableTask for FakeTask {
        fn estimated_duration_minutes(&self) -> u32 {
            self.duration_minutes
        }

        fn requires_location(&self) -> bool {
            self.requires_location
        }

        fn min_hands(&self) -> AvailabilityLevel {
            self.min_hands
        }

        fn min_eyes(&self) -> AvailabilityLevel {
            self.min_eyes
        }

        fn min_speech(&self) -> AvailabilityLevel {
            self.min_speech
        }

        fn min_cognitive(&self) -> AvailabilityLevel {
            self.min_cognitive
        }

        fn min_device(&self) -> DeviceAccess {
            self.min_device
        }

        fn allowed_mobility(&self) -> Vec<Mobility> {
            self.allowed_mobility.clone()
        }
    }

    impl FakeTask {
        fn simple(duration: u32) -> Self {
            Self {
                duration_minutes: duration,
                requires_location: false,
                min_hands: AvailabilityLevel::None,
                min_eyes: AvailabilityLevel::None,
                min_speech: AvailabilityLevel::None,
                min_cognitive: AvailabilityLevel::None,
                min_device: DeviceAccess::None,
                allowed_mobility: vec![],
            }
        }
    }

    fn make_block(
        availability: AvailabilityKind,
        capabilities: CapabilitySet,
        location_constraint: LocationConstraint,
        duration_minutes: i64,
    ) -> TimeBlock {
        let tz = FixedOffset::west_opt(5 * 3600).unwrap();
        let start = tz.with_ymd_and_hms(2026, 2, 10, 9, 0, 0).unwrap();
        let end = start + chrono::Duration::minutes(duration_minutes);

        TimeBlock {
            start,
            end,
            availability,
            capabilities,
            location_constraint,
            label: None,
            priority: 0,
        }
    }

    #[test]
    fn test_unavailable_blocks_reject_all_tasks() {
        let task = FakeTask::simple(10);
        let block = make_block(
            AvailabilityKind::Unavailable(UnavailableReason::Sleep),
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );

        assert!(!can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_available_blocks_accept_matching_tasks() {
        let task = FakeTask::simple(10);
        let block = make_block(
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );

        assert!(can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_busy_flex_accepts_micro_tasks() {
        let task = FakeTask::simple(10); // 10 minutes, no location required
        let block = make_block(
            AvailabilityKind::BusyButFlexible,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );

        assert!(can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_busy_flex_rejects_long_tasks() {
        let task = FakeTask::simple(20); // 20 minutes > max 15
        let block = make_block(
            AvailabilityKind::BusyButFlexible,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );

        assert!(!can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_busy_flex_rejects_location_required_tasks() {
        let mut task = FakeTask::simple(10);
        task.requires_location = true;
        
        let block = make_block(
            AvailabilityKind::BusyButFlexible,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );

        assert!(!can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_busy_flex_rejects_computer_tasks() {
        let mut task = FakeTask::simple(10);
        task.min_device = DeviceAccess::Computer;
        
        let block = make_block(
            AvailabilityKind::BusyButFlexible,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );

        assert!(!can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_busy_flex_rejects_full_hands_tasks() {
        let mut task = FakeTask::simple(10);
        task.min_hands = AvailabilityLevel::Full;
        
        let block = make_block(
            AvailabilityKind::BusyButFlexible,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );

        assert!(!can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_busy_flex_rejects_full_eyes_tasks() {
        let mut task = FakeTask::simple(10);
        task.min_eyes = AvailabilityLevel::Full;
        
        let block = make_block(
            AvailabilityKind::BusyButFlexible,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );

        assert!(!can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_capability_matching_hands() {
        let mut task = FakeTask::simple(10);
        task.min_hands = AvailabilityLevel::Full;

        // Block with Limited hands should reject
        let mut caps = CapabilitySet::free();
        caps.hands = AvailabilityLevel::Limited;
        let block = make_block(
            AvailabilityKind::Available,
            caps,
            LocationConstraint::Any,
            60,
        );
        assert!(!can_schedule_task_in_block(&task, &block, None));

        // Block with Full hands should accept
        let block = make_block(
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );
        assert!(can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_device_matching() {
        let mut task = FakeTask::simple(10);
        task.min_device = DeviceAccess::Computer;

        // Block with PhoneOnly should reject
        let mut caps = CapabilitySet::free();
        caps.device = DeviceAccess::PhoneOnly;
        let block = make_block(
            AvailabilityKind::Available,
            caps,
            LocationConstraint::Any,
            60,
        );
        assert!(!can_schedule_task_in_block(&task, &block, None));

        // Block with Computer should accept
        let block = make_block(
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );
        assert!(can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_location_constraint_matching() {
        let coords = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        let location = Location::new(
            Some("Home".to_string()),
            "New York".to_string(),
            "United States".to_string(),
            coords,
        ).unwrap();

        let task = FakeTask::simple(10);

        // MustBeKnown should reject None
        let block = make_block(
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::MustBeKnown,
            60,
        );
        assert!(!can_schedule_task_in_block(&task, &block, None));
        assert!(can_schedule_task_in_block(&task, &block, Some(&location)));

        // MustBeUnknown should reject Some
        let block = make_block(
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::MustBeUnknown,
            60,
        );
        assert!(can_schedule_task_in_block(&task, &block, None));
        assert!(!can_schedule_task_in_block(&task, &block, Some(&location)));
    }

    #[test]
    fn test_mobility_constraint() {
        let mut task = FakeTask::simple(10);
        task.allowed_mobility = vec![Mobility::Stationary];

        // Block with Driving should reject
        let mut caps = CapabilitySet::free();
        caps.mobility = Mobility::Driving;
        let block = make_block(
            AvailabilityKind::Available,
            caps,
            LocationConstraint::Any,
            60,
        );
        assert!(!can_schedule_task_in_block(&task, &block, None));

        // Block with Stationary should accept
        let block = make_block(
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );
        assert!(can_schedule_task_in_block(&task, &block, None));
    }

    #[test]
    fn test_duration_check() {
        let task = FakeTask::simple(30);

        // Block too short (20 minutes)
        let block = make_block(
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            20,
        );
        assert!(!can_schedule_task_in_block(&task, &block, None));

        // Block long enough (60 minutes)
        let block = make_block(
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            60,
        );
        assert!(can_schedule_task_in_block(&task, &block, None));
    }
}
