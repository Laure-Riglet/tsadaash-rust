use crate::domain::Continents;
use std::fmt;

// ========================================================================
// TIMEZONE VALUE OBJECT
// Encapsulates timezone with domain validation rules
// ========================================================================

/// Represents a validated timezone with continent and city
/// 
/// # Domain Rules
/// - Continent must be a valid IANA timezone continent
/// - City cannot be empty
/// - City must contain only alphanumeric characters, underscores, and slashes
/// 
/// # Examples
/// ```
/// use tsadaash::domain::{Timezone, Continents};
/// 
/// let tz = Timezone::new(Continents::America, "New_York".to_string()).unwrap();
/// assert_eq!(tz.to_iana_string(), "America/New_York");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timezone {
    continent: Continents,
    city: String,
}

impl Timezone {
    /// Creates a new timezone with domain validation
    /// 
    /// # Domain Validation
    /// - City cannot be empty or whitespace-only
    /// - City must contain only alphanumeric, underscore, slash, or hyphen characters
    /// 
    /// # Application Layer Responsibility
    /// The application layer should validate that the city exists in the continent
    /// using the tz_cities.json data (infrastructure concern, not domain rule)
    pub fn new(continent: Continents, city: String) -> Result<Self, TimezoneError> {
        // Domain rule: city cannot be empty
        let trimmed_city = city.trim();
        if trimmed_city.is_empty() {
            return Err(TimezoneError::EmptyCity);
        }
        
        // Domain rule: city must have valid characters for IANA timezone identifiers
        // Valid: alphanumeric, underscore, slash (for sub-zones), hyphen
        if !trimmed_city.chars().all(|c| {
            c.is_alphanumeric() || c == '_' || c == '/' || c == '-'
        }) {
            return Err(TimezoneError::InvalidCityFormat(city.clone()));
        }
        
        Ok(Self {
            continent,
            city: trimmed_city.to_string(),
        })
    }
    
    /// Returns the continent part of the timezone
    pub fn continent(&self) -> Continents {
        self.continent
    }
    
    /// Returns the city part of the timezone
    pub fn city(&self) -> &str {
        &self.city
    }
    
    /// Returns the full IANA timezone identifier (e.g., "America/New_York")
    /// This is the format expected by datetime libraries like chrono-tz
    pub fn to_iana_string(&self) -> String {
        format!("{}/{}", self.continent, self.city)
    }
}

impl fmt::Display for Timezone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iana_string())
    }
}

// ========================================================================
// TIMEZONE ERRORS
// ========================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimezoneError {
    /// City name is empty or whitespace-only
    EmptyCity,
    
    /// City contains invalid characters
    /// Valid characters: alphanumeric, underscore, slash, hyphen
    InvalidCityFormat(String),
}

impl fmt::Display for TimezoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimezoneError::EmptyCity => {
                write!(f, "Timezone city cannot be empty")
            }
            TimezoneError::InvalidCityFormat(city) => {
                write!(
                    f,
                    "Invalid timezone city format: '{}'. Must contain only alphanumeric characters, underscores, slashes, or hyphens",
                    city
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
    fn test_valid_timezone_creation() {
        let tz = Timezone::new(Continents::America, "New_York".to_string()).unwrap();
        assert_eq!(tz.continent(), Continents::America);
        assert_eq!(tz.city(), "New_York");
        assert_eq!(tz.to_iana_string(), "America/New_York");
    }

    #[test]
    fn test_timezone_with_subzone() {
        let tz = Timezone::new(Continents::America, "Argentina/Buenos_Aires".to_string()).unwrap();
        assert_eq!(tz.to_iana_string(), "America/Argentina/Buenos_Aires");
    }

    #[test]
    fn test_timezone_with_hyphen() {
        let tz = Timezone::new(Continents::Atlantic, "Cape_Verde".to_string()).unwrap();
        assert_eq!(tz.city(), "Cape_Verde");
    }

    #[test]
    fn test_empty_city_error() {
        let result = Timezone::new(Continents::Europe, "".to_string());
        assert_eq!(result, Err(TimezoneError::EmptyCity));
    }

    #[test]
    fn test_whitespace_only_city_error() {
        let result = Timezone::new(Continents::Europe, "   ".to_string());
        assert_eq!(result, Err(TimezoneError::EmptyCity));
    }

    #[test]
    fn test_invalid_city_format() {
        let result = Timezone::new(Continents::America, "New York".to_string()); // Space not allowed
        assert!(matches!(result, Err(TimezoneError::InvalidCityFormat(_))));
    }

    #[test]
    fn test_timezone_trimming() {
        let tz = Timezone::new(Continents::Asia, "  Tokyo  ".to_string()).unwrap();
        assert_eq!(tz.city(), "Tokyo"); // Should be trimmed
    }

    #[test]
    fn test_timezone_display() {
        let tz = Timezone::new(Continents::Europe, "Paris".to_string()).unwrap();
        assert_eq!(tz.to_string(), "Europe/Paris");
    }

    #[test]
    fn test_timezone_clone_and_eq() {
        let tz1 = Timezone::new(Continents::Asia, "Tokyo".to_string()).unwrap();
        let tz2 = tz1.clone();
        assert_eq!(tz1, tz2);
    }
}
