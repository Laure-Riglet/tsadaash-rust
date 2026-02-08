use crate::domain::entities::user::Location;
use crate::config;

// ========================================================================
// AVAILABILITY TYPES
// ========================================================================

/// Represents the availability status during a time period
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvailabilityKind {
    /// User is not available for tasks
    Unavailable(UnavailableReason),
    /// User is busy but can handle short, low-friction tasks
    BusyButFlexible,
    /// User is available for tasks
    Available,
}

/// Reason for unavailability (for logging/display purposes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnavailableReason {
    Sleep,
    Work,
    Appointment,
    Focus,
    Other(String),
}

// ========================================================================
// CAPABILITY MODELING
// ========================================================================

/// Represents the level of availability for a capability (hands, eyes, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AvailabilityLevel {
    None = 0,
    Limited = 1,
    Full = 2,
}

/// Device access level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeviceAccess {
    None = 0,
    PhoneOnly = 1,
    Computer = 2,
}

/// Mobility status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mobility {
    Stationary,
    InTransit,
    Driving,
}

/// Represents the full set of capabilities available during a time period
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySet {
    pub hands: AvailabilityLevel,
    pub eyes: AvailabilityLevel,
    pub speech: AvailabilityLevel,
    pub cognitive: AvailabilityLevel,
    pub device: DeviceAccess,
    pub mobility: Mobility,
}

impl CapabilitySet {
    /// Maximum capabilities - user is free and at a computer
    pub fn free() -> Self {
        Self {
            hands: AvailabilityLevel::Full,
            eyes: AvailabilityLevel::Full,
            speech: AvailabilityLevel::Full,
            cognitive: AvailabilityLevel::Full,
            device: DeviceAccess::Computer,
            mobility: Mobility::Stationary,
        }
    }

    /// Driving - hands and eyes unavailable, limited cognitive, phone/no device, driving
    pub fn driving() -> Self {
        Self {
            hands: AvailabilityLevel::None,
            eyes: AvailabilityLevel::None,
            speech: AvailabilityLevel::Full,
            cognitive: AvailabilityLevel::Limited,
            device: DeviceAccess::None,
            mobility: Mobility::Driving,
        }
    }

    /// In transit (e.g., passenger in vehicle, walking) - limited hands/eyes, phone available
    pub fn in_transit() -> Self {
        Self {
            hands: AvailabilityLevel::Limited,
            eyes: AvailabilityLevel::Limited,
            speech: AvailabilityLevel::Full,
            cognitive: AvailabilityLevel::Full,
            device: DeviceAccess::PhoneOnly,
            mobility: Mobility::InTransit,
        }
    }
}

// ========================================================================
// LOCATION CONSTRAINTS
// ========================================================================

/// Constraint on location for a time period
#[derive(Debug, Clone, PartialEq)]
pub enum LocationConstraint {
    /// Any location is acceptable (or location doesn't matter)
    Any,
    /// Must have a known location
    MustBeKnown,
    /// Must be in an unknown location
    MustBeUnknown,
    /// Must be in one of the specified locations
    MustBeOneOf(Vec<Location>),
}

impl LocationConstraint {
    /// Check if a given location satisfies this constraint
    pub fn matches(&self, current_location: Option<&Location>) -> bool {
        match self {
            LocationConstraint::Any => true,
            LocationConstraint::MustBeKnown => current_location.is_some(),
            LocationConstraint::MustBeUnknown => current_location.is_none(),
            LocationConstraint::MustBeOneOf(allowed) => {
                if let Some(loc) = current_location {
                    allowed.iter().any(|allowed_loc| allowed_loc == loc)
                } else {
                    false
                }
            }
        }
    }
}

// ========================================================================
// CONSTANTS
// ========================================================================

/// Maximum task duration (in minutes) allowed during BusyButFlexible periods
pub fn busy_flex_max_minutes() -> u32 {
    config::schedule_busy_flex_max_minutes()
}

/// Maximum hands level allowed during BusyButFlexible periods
pub fn busy_flex_max_hands() -> AvailabilityLevel {
    match config::schedule_busy_flex_max_hands_level() {
        0 => AvailabilityLevel::None,
        1 => AvailabilityLevel::Limited,
        _ => AvailabilityLevel::Full,
    }
}

/// Maximum eyes level allowed during BusyButFlexible periods
pub fn busy_flex_max_eyes() -> AvailabilityLevel {
    match config::schedule_busy_flex_max_eyes_level() {
        0 => AvailabilityLevel::None,
        1 => AvailabilityLevel::Limited,
        _ => AvailabilityLevel::Full,
    }
}

/// Maximum device required for BusyButFlexible periods
pub fn busy_flex_max_device() -> DeviceAccess {
    match config::schedule_busy_flex_max_device_level() {
        0 => DeviceAccess::None,
        1 => DeviceAccess::PhoneOnly,
        _ => DeviceAccess::Computer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::user::GeoCoordinates;

    #[test]
    fn test_availability_level_ordering() {
        assert!(AvailabilityLevel::None < AvailabilityLevel::Limited);
        assert!(AvailabilityLevel::Limited < AvailabilityLevel::Full);
    }

    #[test]
    fn test_device_access_ordering() {
        assert!(DeviceAccess::None < DeviceAccess::PhoneOnly);
        assert!(DeviceAccess::PhoneOnly < DeviceAccess::Computer);
    }

    #[test]
    fn test_capability_presets() {
        let free = CapabilitySet::free();
        assert_eq!(free.hands, AvailabilityLevel::Full);
        assert_eq!(free.device, DeviceAccess::Computer);

        let driving = CapabilitySet::driving();
        assert_eq!(driving.hands, AvailabilityLevel::None);
        assert_eq!(driving.mobility, Mobility::Driving);

        let transit = CapabilitySet::in_transit();
        assert_eq!(transit.hands, AvailabilityLevel::Limited);
        assert_eq!(transit.device, DeviceAccess::PhoneOnly);
    }

    #[test]
    fn test_location_constraint_any() {
        let constraint = LocationConstraint::Any;
        assert!(constraint.matches(None));
        
        let coords = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        let location = Location::new(
            Some("Home".to_string()),
            "New York".to_string(),
            "United States".to_string(),
            coords,
        ).unwrap();
        assert!(constraint.matches(Some(&location)));
    }

    #[test]
    fn test_location_constraint_must_be_known() {
        let constraint = LocationConstraint::MustBeKnown;
        assert!(!constraint.matches(None));

        let coords = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        let location = Location::new(
            Some("Home".to_string()),
            "New York".to_string(),
            "United States".to_string(),
            coords,
        ).unwrap();
        assert!(constraint.matches(Some(&location)));
    }

    #[test]
    fn test_location_constraint_must_be_unknown() {
        let constraint = LocationConstraint::MustBeUnknown;
        assert!(constraint.matches(None));

        let coords = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        let location = Location::new(
            Some("Home".to_string()),
            "New York".to_string(),
            "United States".to_string(),
            coords,
        ).unwrap();
        assert!(!constraint.matches(Some(&location)));
    }

    #[test]
    fn test_location_constraint_must_be_one_of() {
        let coords1 = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        let home = Location::new(
            Some("Home".to_string()),
            "New York".to_string(),
            "United States".to_string(),
            coords1,
        ).unwrap();

        let coords2 = GeoCoordinates::new(51.5074, -0.1278).unwrap();
        let work = Location::new(
            Some("Work".to_string()),
            "London".to_string(),
            "United Kingdom".to_string(),
            coords2,
        ).unwrap();

        let constraint = LocationConstraint::MustBeOneOf(vec![home.clone(), work.clone()]);
        
        // Should reject None
        assert!(!constraint.matches(None));
        
        // Should accept matching location
        assert!(constraint.matches(Some(&home)));
        assert!(constraint.matches(Some(&work)));
        
        // Should reject non-matching location
        let coords3 = GeoCoordinates::new(48.8566, 2.3522).unwrap();
        let other = Location::new(
            Some("Other".to_string()),
            "Paris".to_string(),
            "France".to_string(),
            coords3,
        ).unwrap();
        assert!(!constraint.matches(Some(&other)));
    }
}
