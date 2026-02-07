use std::fmt;

// ========================================================================
// TIMEZONE VALUE OBJECT
// Encapsulates IANA timezone identifier with format validation only
// ========================================================================

/// Represents a timezone identifier (e.g., "America/New_York", "Europe/Paris")
/// 
/// # Domain Rules (Format Only)
/// - Must be in "Area/Location" format (e.g., "America/New_York")
/// - Can have sub-zones (e.g., "America/Argentina/Buenos_Aires")
/// - Characters must be alphanumeric, underscore, slash, hyphen, or plus
/// 
/// # Application Layer Responsibility
/// The application layer should validate that the timezone actually exists
/// using the tz_cities.json data or chrono-tz crate (infrastructure concern)
/// 
/// # Examples
/// ```
/// use tsadaash::domain::Timezone;
/// 
/// let tz = Timezone::new("America/New_York".to_string()).unwrap();
/// assert_eq!(tz.as_str(), "America/New_York");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timezone {
    identifier: String,
}

impl Timezone {
    /// Creates a new timezone with domain validation (format only)
    /// 
    /// # Domain Validation
    /// - Identifier cannot be empty
    /// - Must contain at least one slash (Area/Location format)
    /// - Must contain only valid IANA timezone characters
    /// 
    /// # Examples
    /// ```
    /// use tsadaash::domain::Timezone;
    /// 
    /// // Valid
    /// assert!(Timezone::new("Europe/Paris".to_string()).is_ok());
    /// assert!(Timezone::new("America/Argentina/Buenos_Aires".to_string()).is_ok());
    /// 
    /// // Invalid format
    /// assert!(Timezone::new("Paris".to_string()).is_err());  // Missing area
    /// assert!(Timezone::new("Europe/".to_string()).is_err()); // Empty location
    /// assert!(Timezone::new("".to_string()).is_err());        // Empty
    /// ```
    pub fn new(identifier: String) -> Result<Self, TimezoneError> {
        let trimmed = identifier.trim();
        
        // Domain rule: cannot be empty
        if trimmed.is_empty() {
            return Err(TimezoneError::EmptyIdentifier);
        }
        
        // Domain rule: must have Area/Location format (at least one slash)
        if !trimmed.contains('/') {
            return Err(TimezoneError::MissingAreaSeparator(identifier.clone()));
        }
        
        // Domain rule: must not start or end with slash
        if trimmed.starts_with('/') || trimmed.ends_with('/') {
            return Err(TimezoneError::InvalidFormat(identifier.clone()));
        }
        
        // Domain rule: must have valid IANA timezone characters
        // Valid: alphanumeric, underscore, slash, hyphen, plus
        if !trimmed.chars().all(|c| {
            c.is_alphanumeric() || c == '_' || c == '/' || c == '-' || c == '+'
        }) {
            return Err(TimezoneError::InvalidCharacters(identifier.clone()));
        }
        
        // Domain rule: no part can be empty (no double slashes)
        if trimmed.contains("//") {
            return Err(TimezoneError::InvalidFormat(identifier.clone()));
        }
        
        Ok(Self {
            identifier: trimmed.to_string(),
        })
    }
    
    /// Returns the timezone identifier as a string slice
    pub fn as_str(&self) -> &str {
        &self.identifier
    }
    
    /// Converts into the owned String
    pub fn into_string(self) -> String {
        self.identifier
    }
}

impl fmt::Display for Timezone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier)
    }
}

// ========================================================================
// TIMEZONE ERRORS
// ========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimezoneError {
    /// Timezone identifier is empty or whitespace-only
    EmptyIdentifier,
    
    /// Timezone identifier doesn't contain '/' (missing Area/Location format)
    MissingAreaSeparator(String),
    
    /// Timezone identifier has invalid format (e.g., starts/ends with slash, double slashes)
    InvalidFormat(String),
    
    /// Timezone identifier contains invalid characters
    /// Valid characters: alphanumeric, underscore, slash, hyphen, plus
    InvalidCharacters(String),
}

impl fmt::Display for TimezoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimezoneError::EmptyIdentifier => {
                write!(f, "Timezone identifier cannot be empty")
            }
            TimezoneError::MissingAreaSeparator(tz) => {
                write!(
                    f,
                    "Invalid timezone '{}': must be in 'Area/Location' format (e.g., 'America/New_York')",
                    tz
                )
            }
            TimezoneError::InvalidFormat(tz) => {
                write!(
                    f,
                    "Invalid timezone format '{}': cannot start/end with slash or contain empty parts",
                    tz
                )
            }
            TimezoneError::InvalidCharacters(tz) => {
                write!(
                    f,
                    "Invalid timezone '{}': must contain only alphanumeric characters, underscores, slashes, hyphens, or plus signs",
                    tz
                )
            }
        }
    }
}

impl std::error::Error for TimezoneError {}

// ========================================================================
// TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_timezone_simple() {
        let tz = Timezone::new("America/New_York".to_string()).unwrap();
        assert_eq!(tz.as_str(), "America/New_York");
        assert_eq!(tz.to_string(), "America/New_York");
    }

    #[test]
    fn test_valid_timezone_with_subzone() {
        let tz = Timezone::new("America/Argentina/Buenos_Aires".to_string()).unwrap();
        assert_eq!(tz.as_str(), "America/Argentina/Buenos_Aires");
    }

    #[test]
    fn test_valid_timezone_with_hyphen() {
        let tz = Timezone::new("America/Port-au-Prince".to_string()).unwrap();
        assert_eq!(tz.as_str(), "America/Port-au-Prince");
    }

    #[test]
    fn test_valid_timezone_with_plus() {
        let tz = Timezone::new("Etc/GMT+5".to_string()).unwrap();
        assert_eq!(tz.as_str(), "Etc/GMT+5");
    }

    #[test]
    fn test_empty_identifier_error() {
        let result = Timezone::new("".to_string());
        assert_eq!(result, Err(TimezoneError::EmptyIdentifier));
    }

    #[test]
    fn test_whitespace_only_error() {
        let result = Timezone::new("   ".to_string());
        assert_eq!(result, Err(TimezoneError::EmptyIdentifier));
    }

    #[test]
    fn test_missing_slash_error() {
        let result = Timezone::new("NewYork".to_string());
        assert!(matches!(result, Err(TimezoneError::MissingAreaSeparator(_))));
    }

    #[test]
    fn test_starts_with_slash_error() {
        let result = Timezone::new("/America/New_York".to_string());
        assert!(matches!(result, Err(TimezoneError::InvalidFormat(_))));
    }

    #[test]
    fn test_ends_with_slash_error() {
        let result = Timezone::new("America/New_York/".to_string());
        assert!(matches!(result, Err(TimezoneError::InvalidFormat(_))));
    }

    #[test]
    fn test_double_slash_error() {
        let result = Timezone::new("America//New_York".to_string());
        assert!(matches!(result, Err(TimezoneError::InvalidFormat(_))));
    }

    #[test]
    fn test_invalid_characters_space() {
        let result = Timezone::new("America/New York".to_string());
        assert!(matches!(result, Err(TimezoneError::InvalidCharacters(_))));
    }

    #[test]
    fn test_invalid_characters_special() {
        let result = Timezone::new("America/New@York".to_string());
        assert!(matches!(result, Err(TimezoneError::InvalidCharacters(_))));
    }

    #[test]
    fn test_timezone_trimming() {
        let tz = Timezone::new("  Europe/Paris  ".to_string()).unwrap();
        assert_eq!(tz.as_str(), "Europe/Paris");
    }

    #[test]
    fn test_timezone_clone_and_eq() {
        let tz1 = Timezone::new("Asia/Tokyo".to_string()).unwrap();
        let tz2 = tz1.clone();
        assert_eq!(tz1, tz2);
    }

    #[test]
    fn test_into_string() {
        let tz = Timezone::new("Europe/London".to_string()).unwrap();
        let s = tz.into_string();
        assert_eq!(s, "Europe/London");
    }

    #[test]
    fn test_accepts_any_area_location_pair() {
        // Domain doesn't care if these are real - just that format is valid
        assert!(Timezone::new("FakeContinent/FakeCity".to_string()).is_ok());
        assert!(Timezone::new("Mars/Olympus_Mons".to_string()).is_ok());
    }
}
