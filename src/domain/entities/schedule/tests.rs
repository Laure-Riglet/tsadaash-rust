// Integration tests for the schedule module
// These tests verify end-to-end behavior across all components

#[cfg(test)]
mod integration_tests {
    use crate::domain::entities::schedule::{
        expansion::expand_template,
        matching::{can_schedule_task_in_block, SchedulableTask},
        template::{RecurringRule, ScheduleTemplate},
        types::{
            AvailabilityKind, AvailabilityLevel, CapabilitySet, DeviceAccess,
            LocationConstraint, Mobility, UnavailableReason,
        },
    };
    use crate::domain::entities::user::{GeoCoordinates, Location};
    use chrono::{FixedOffset, NaiveTime, TimeZone, Timelike, Weekday};

    // ========================================================================
    // TEST HELPERS
    // ========================================================================

    struct TestTask {
        duration_minutes: u32,
        requires_location: bool,
        min_hands: AvailabilityLevel,
        min_eyes: AvailabilityLevel,
        min_speech: AvailabilityLevel,
        min_cognitive: AvailabilityLevel,
        min_device: DeviceAccess,
        allowed_mobility: Vec<Mobility>,
    }

    impl SchedulableTask for TestTask {
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

    impl TestTask {
        fn new_simple(duration_minutes: u32) -> Self {
            Self {
                duration_minutes,
                requires_location: false,
                min_hands: AvailabilityLevel::None,
                min_eyes: AvailabilityLevel::None,
                min_speech: AvailabilityLevel::None,
                min_cognitive: AvailabilityLevel::None,
                min_device: DeviceAccess::None,
                allowed_mobility: vec![],
            }
        }

        fn new_micro() -> Self {
            Self::new_simple(10)
        }

        fn new_computer_task(duration_minutes: u32) -> Self {
            Self {
                duration_minutes,
                requires_location: false,
                min_hands: AvailabilityLevel::Full,
                min_eyes: AvailabilityLevel::Full,
                min_speech: AvailabilityLevel::None,
                min_cognitive: AvailabilityLevel::Full,
                min_device: DeviceAccess::Computer,
                allowed_mobility: vec![Mobility::Stationary],
            }
        }
    }

    // ========================================================================
    // SCENARIO 1: Weekly Work Schedule with Lunch Break
    // ========================================================================

    #[test]
    fn test_typical_work_week_schedule() {
        // Create a typical 9-5 work schedule Mon-Fri with lunch break
        let work_rule = RecurringRule::new(
            vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            AvailabilityKind::BusyButFlexible,
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Work".to_string()),
            0,
        )
        .unwrap();

        let lunch_rule = RecurringRule::new(
            vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Lunch".to_string()),
            10, // Higher priority to override work
        )
        .unwrap();

        let template = ScheduleTemplate::new(
            1,
            1,
            "Work Week".to_string(),
            "America/New_York".to_string(),
            vec![work_rule, lunch_rule],
        )
        .unwrap();

        // Expand for Tuesday Feb 10, 2026
        let tz = FixedOffset::west_opt(5 * 3600).unwrap();
        let start = tz.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap();
        let end = tz.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap();

        let blocks = expand_template(&template, start, end);

        // Should have 3 blocks: 9-12 Work, 12-13 Lunch, 13-17 Work
        assert_eq!(blocks.len(), 3);

        assert_eq!(blocks[0].label, Some("Work".to_string()));
        assert!(matches!(blocks[0].availability, AvailabilityKind::BusyButFlexible));

        assert_eq!(blocks[1].label, Some("Lunch".to_string()));
        assert!(matches!(blocks[1].availability, AvailabilityKind::Available));

        assert_eq!(blocks[2].label, Some("Work".to_string()));
        assert!(matches!(blocks[2].availability, AvailabilityKind::BusyButFlexible));

        // Test that micro tasks can be scheduled during work hours
        let micro_task = TestTask::new_micro();
        assert!(can_schedule_task_in_block(&micro_task, &blocks[0], None));

        // Test that computer tasks can be scheduled during lunch
        let computer_task = TestTask::new_computer_task(30);
        assert!(can_schedule_task_in_block(&computer_task, &blocks[1], None));

        // Test that computer tasks cannot be scheduled during work hours (busy-but-flexible)
        assert!(!can_schedule_task_in_block(&computer_task, &blocks[0], None));
    }

    // ========================================================================
    // SCENARIO 2: Sleep Schedule (Overnight Rule)
    // ========================================================================

    #[test]
    fn test_overnight_sleep_schedule() {
        let sleep_rule = RecurringRule::new(
            vec![
                Weekday::Sun,
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
            ],
            NaiveTime::from_hms_opt(23, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            AvailabilityKind::Unavailable(UnavailableReason::Sleep),
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Sleep".to_string()),
            0,
        )
        .unwrap();

        let template = ScheduleTemplate::new(
            1,
            1,
            "Sleep Schedule".to_string(),
            "America/New_York".to_string(),
            vec![sleep_rule],
        )
        .unwrap();

        // Expand for 2 days
        let tz = FixedOffset::west_opt(5 * 3600).unwrap();
        let start = tz.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap();
        let end = tz.with_ymd_and_hms(2026, 2, 12, 0, 0, 0).unwrap();

        let blocks = expand_template(&template, start, end);

        // Should have multiple sleep blocks (overnight periods)
        assert!(blocks.len() >= 2);

        // All blocks should be Unavailable
        for block in &blocks {
            assert!(matches!(
                block.availability,
                AvailabilityKind::Unavailable(UnavailableReason::Sleep)
            ));
            assert_eq!(block.label, Some("Sleep".to_string()));
        }

        // No tasks should be schedulable during sleep
        let task = TestTask::new_simple(10);
        for block in &blocks {
            assert!(!can_schedule_task_in_block(&task, block, None));
        }
    }

    // ========================================================================
    // SCENARIO 3: Commute Schedule with Limited Capabilities
    // ========================================================================

    #[test]
    fn test_commute_with_limited_capabilities() {
        let commute_rule = RecurringRule::new(
            vec![Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri],
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            AvailabilityKind::BusyButFlexible,
            CapabilitySet::in_transit(),
            LocationConstraint::Any,
            Some("Commute".to_string()),
            0,
        )
        .unwrap();

        let template = ScheduleTemplate::new(
            1,
            1,
            "Commute".to_string(),
            "America/New_York".to_string(),
            vec![commute_rule],
        )
        .unwrap();

        let tz = FixedOffset::west_opt(5 * 3600).unwrap();
        let start = tz.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap(); // Tuesday
        let end = tz.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap();

        let blocks = expand_template(&template, start, end);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].capabilities.mobility, Mobility::InTransit);
        assert_eq!(blocks[0].capabilities.hands, AvailabilityLevel::Limited);
        assert_eq!(blocks[0].capabilities.device, DeviceAccess::PhoneOnly);

        // Micro task with phone should work
        let phone_task = TestTask {
            duration_minutes: 10,
            requires_location: false,
            min_hands: AvailabilityLevel::Limited,
            min_eyes: AvailabilityLevel::Limited,
            min_speech: AvailabilityLevel::None,
            min_cognitive: AvailabilityLevel::None,
            min_device: DeviceAccess::PhoneOnly,
            allowed_mobility: vec![Mobility::InTransit],
        };

        assert!(can_schedule_task_in_block(&phone_task, &blocks[0], None));

        // Computer task should not work
        let computer_task = TestTask::new_computer_task(10);
        assert!(!can_schedule_task_in_block(&computer_task, &blocks[0], None));
    }

    // ========================================================================
    // SCENARIO 4: Location-Based Availability
    // ========================================================================

    #[test]
    fn test_location_based_availability() {
        let coords_home = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        let home = Location::new(
            Some("Home".to_string()),
            "New York".to_string(),
            "United States".to_string(),
            coords_home,
        )
        .unwrap();

        let coords_work = GeoCoordinates::new(40.7589, -73.9851).unwrap();
        let work = Location::new(
            Some("Work".to_string()),
            "New York".to_string(),
            "United States".to_string(),
            coords_work,
        )
        .unwrap();

        // Rule that requires being at home or work
        let rule = RecurringRule::new(
            vec![Weekday::Tue],
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::MustBeOneOf(vec![home.clone(), work.clone()]),
            Some("At Known Location".to_string()),
            0,
        )
        .unwrap();

        let template = ScheduleTemplate::new(
            1,
            1,
            "Location Test".to_string(),
            "America/New_York".to_string(),
            vec![rule],
        )
        .unwrap();

        let tz = FixedOffset::west_opt(5 * 3600).unwrap();
        let start = tz.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap();
        let end = tz.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap();

        let blocks = expand_template(&template, start, end);
        assert_eq!(blocks.len(), 1);

        let task = TestTask::new_simple(30);

        // Should work at home
        assert!(can_schedule_task_in_block(&task, &blocks[0], Some(&home)));

        // Should work at work
        assert!(can_schedule_task_in_block(&task, &blocks[0], Some(&work)));

        // Should not work at unknown location
        assert!(!can_schedule_task_in_block(&task, &blocks[0], None));

        // Should not work at different location
        let coords_other = GeoCoordinates::new(51.5074, -0.1278).unwrap();
        let other = Location::new(
            Some("Elsewhere".to_string()),
            "London".to_string(),
            "United Kingdom".to_string(),
            coords_other,
        )
        .unwrap();
        assert!(!can_schedule_task_in_block(&task, &blocks[0], Some(&other)));
    }

    // ========================================================================
    // SCENARIO 5: Complex Priority Resolution
    // ========================================================================

    #[test]
    fn test_multiple_priority_overlaps() {
        // Base availability: All day available
        let base = RecurringRule::new(
            vec![Weekday::Tue],
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(23, 59, 0).unwrap(),
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Available".to_string()),
            0,
        )
        .unwrap();

        // Work hours: 9-17 busy but flexible
        let work = RecurringRule::new(
            vec![Weekday::Tue],
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            AvailabilityKind::BusyButFlexible,
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Work".to_string()),
            5,
        )
        .unwrap();

        // Meeting: 10-11 unavailable
        let meeting = RecurringRule::new(
            vec![Weekday::Tue],
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            AvailabilityKind::Unavailable(UnavailableReason::Appointment),
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Meeting".to_string()),
            10,
        )
        .unwrap();

        // Lunch: 12-13 available
        let lunch = RecurringRule::new(
            vec![Weekday::Tue],
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Lunch".to_string()),
            10,
        )
        .unwrap();

        let template = ScheduleTemplate::new(
            1,
            1,
            "Complex Schedule".to_string(),
            "America/New_York".to_string(),
            vec![base, work, meeting, lunch],
        )
        .unwrap();

        let tz = FixedOffset::west_opt(5 * 3600).unwrap();
        let start = tz.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap();
        let end = tz.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap();

        let blocks = expand_template(&template, start, end);

        // Verify the schedule structure
        let mut found_meeting = false;
        let mut found_lunch = false;
        let mut found_work = false;

        for block in &blocks {
            if block.label == Some("Meeting".to_string()) {
                found_meeting = true;
                assert!(matches!(
                    block.availability,
                    AvailabilityKind::Unavailable(_)
                ));
            }
            if block.label == Some("Lunch".to_string()) {
                found_lunch = true;
                assert!(matches!(block.availability, AvailabilityKind::Available));
            }
            if block.label == Some("Work".to_string()) {
                found_work = true;
                assert!(matches!(
                    block.availability,
                    AvailabilityKind::BusyButFlexible
                ));
            }
        }

        assert!(found_meeting, "Should have meeting block");
        assert!(found_lunch, "Should have lunch block");
        assert!(found_work, "Should have work blocks");
    }

    // ========================================================================
    // SCENARIO 6: Tie-Breaking (Same Priority)
    // ========================================================================

    #[test]
    fn test_same_priority_tie_breaking() {
        // Two rules with same priority overlapping
        let rule1 = RecurringRule::new(
            vec![Weekday::Tue],
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            AvailabilityKind::Available,
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Rule1".to_string()),
            5,
        )
        .unwrap();

        let rule2 = RecurringRule::new(
            vec![Weekday::Tue],
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            AvailabilityKind::Unavailable(UnavailableReason::Work),
            CapabilitySet::free(),
            LocationConstraint::Any,
            Some("Rule2".to_string()),
            5, // Same priority
        )
        .unwrap();

        let template = ScheduleTemplate::new(
            1,
            1,
            "Tie Break Test".to_string(),
            "America/New_York".to_string(),
            vec![rule1, rule2],
        )
        .unwrap();

        let tz = FixedOffset::west_opt(5 * 3600).unwrap();
        let start = tz.with_ymd_and_hms(2026, 2, 10, 0, 0, 0).unwrap();
        let end = tz.with_ymd_and_hms(2026, 2, 11, 0, 0, 0).unwrap();

        let blocks = expand_template(&template, start, end);

        // Should have at least one block
        assert!(!blocks.is_empty());

        // The 10-11 slot should be Unavailable (more restrictive wins)
        let middle_block = blocks
            .iter()
            .find(|b| b.start.hour() == 10 || b.label == Some("Rule2".to_string()));

        if let Some(block) = middle_block {
            assert!(matches!(
                block.availability,
                AvailabilityKind::Unavailable(_)
            ));
        }
    }
}
