use std::fmt;

// ========================================================================
// LOCATION VALUE OBJECT
// Represents a physical location with geographic coordinates
// ========================================================================

/// Represents a user's location with geographic information
/// 
/// # Domain Rules
/// - City and country are required
/// - Name is optional (e.g., "Home", "Office")
/// - Coordinates must be valid (lat: -90 to 90, lng: -180 to 180)
/// 
/// # Examples
/// ```
/// use tsadaash::domain::{Location, GeoCoordinates};
/// 
/// let coords = GeoCoordinates::new(40.7128, -74.0060).unwrap();
/// let location = Location::new(
///     Some("Home".to_string()),
///     "New York".to_string(),
///     "United States".to_string(),
///     coords,
/// ).unwrap();
/// 
/// assert_eq!(location.city(), "New York");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    name: Option<String>,
    city: String,
    country: String,
    geoloc: GeoCoordinates,
}

impl Location {
    /// Creates a new location with validation
    /// 
    /// # Domain Validation
    /// - City cannot be empty or whitespace-only
    /// - Country cannot be empty or whitespace-only
    /// - Name (if provided) cannot be empty or whitespace-only
    /// - Coordinates must be valid
    pub fn new(
        name: Option<String>,
        city: String,
        country: String,
        geoloc: GeoCoordinates,
    ) -> Result<Self, LocationError> {
        // Validate city
        let trimmed_city = city.trim();
        if trimmed_city.is_empty() {
            return Err(LocationError::EmptyCity);
        }
        
        // Validate country
        let trimmed_country = country.trim();
        if trimmed_country.is_empty() {
            return Err(LocationError::EmptyCountry);
        }
        
        // Validate name if provided
        let validated_name = if let Some(n) = name {
            let trimmed_name = n.trim();
            if trimmed_name.is_empty() {
                return Err(LocationError::EmptyName);
            }
            Some(trimmed_name.to_string())
        } else {
            None
        };
        
        Ok(Self {
            name: validated_name,
            city: trimmed_city.to_string(),
            country: trimmed_country.to_string(),
            geoloc,
        })
    }
    
    /// Returns the optional location name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    /// Returns the city
    pub fn city(&self) -> &str {
        &self.city
    }
    
    /// Returns the country
    pub fn country(&self) -> &str {
        &self.country
    }
    
    /// Returns the geographic coordinates
    pub fn geoloc(&self) -> &GeoCoordinates {
        &self.geoloc
    }
    
    /// Updates the location name
    pub fn set_name(&mut self, name: Option<String>) -> Result<(), LocationError> {
        if let Some(n) = name {
            let trimmed = n.trim();
            if trimmed.is_empty() {
                return Err(LocationError::EmptyName);
            }
            self.name = Some(trimmed.to_string());
        } else {
            self.name = None;
        }
        Ok(())
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{} ({}, {})", name, self.city, self.country)
        } else {
            write!(f, "{}, {}", self.city, self.country)
        }
    }
}

// ========================================================================
// GEOGRAPHIC COORDINATES VALUE OBJECT
// ========================================================================

/// Represents validated geographic coordinates (latitude and longitude)
/// 
/// # Domain Rules
/// - Latitude must be between -90.0 and 90.0 (inclusive)
/// - Longitude must be between -180.0 and 180.0 (inclusive)
/// 
/// # Examples
/// ```
/// use tsadaash::domain::GeoCoordinates;
/// 
/// // Valid coordinates
/// let nyc = GeoCoordinates::new(40.7128, -74.0060).unwrap();
/// assert_eq!(nyc.latitude(), 40.7128);
/// assert_eq!(nyc.longitude(), -74.0060);
/// 
/// // Invalid coordinates
/// assert!(GeoCoordinates::new(91.0, 0.0).is_err());  // Latitude too high
/// assert!(GeoCoordinates::new(0.0, 181.0).is_err()); // Longitude too high
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoCoordinates {
    latitude: f64,
    longitude: f64,
}

impl GeoCoordinates {
    /// Creates new geographic coordinates with validation
    /// 
    /// # Arguments
    /// * `latitude` - Latitude in decimal degrees (-90.0 to 90.0)
    /// * `longitude` - Longitude in decimal degrees (-180.0 to 180.0)
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, GeoCoordinatesError> {
        // Validate latitude range
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(GeoCoordinatesError::InvalidLatitude(latitude));
        }
        
        // Validate longitude range
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(GeoCoordinatesError::InvalidLongitude(longitude));
        }
        
        // Check for NaN or infinity
        if !latitude.is_finite() {
            return Err(GeoCoordinatesError::InvalidLatitude(latitude));
        }
        if !longitude.is_finite() {
            return Err(GeoCoordinatesError::InvalidLongitude(longitude));
        }
        
        Ok(Self { latitude, longitude })
    }
    
    /// Returns the latitude in decimal degrees
    pub fn latitude(&self) -> f64 {
        self.latitude
    }
    
    /// Returns the longitude in decimal degrees
    pub fn longitude(&self) -> f64 {
        self.longitude
    }
    
    /// Returns coordinates as a tuple (latitude, longitude)
    pub fn as_tuple(&self) -> (f64, f64) {
        (self.latitude, self.longitude)
    }
}

impl fmt::Display for GeoCoordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}°, {:.4}°", self.latitude, self.longitude)
    }
}

// ========================================================================
// ERRORS
// ========================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum LocationError {
    /// Location name is empty or whitespace-only (when provided)
    EmptyName,
    
    /// City is empty or whitespace-only
    EmptyCity,
    
    /// Country is empty or whitespace-only
    EmptyCountry,
}

impl fmt::Display for LocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationError::EmptyName => {
                write!(f, "Location name cannot be empty when provided")
            }
            LocationError::EmptyCity => {
                write!(f, "City cannot be empty")
            }
            LocationError::EmptyCountry => {
                write!(f, "Country cannot be empty")
            }
        }
    }
}

impl std::error::Error for LocationError {}

#[derive(Debug, Clone, PartialEq)]
pub enum GeoCoordinatesError {
    /// Latitude is out of valid range (-90 to 90)
    InvalidLatitude(f64),
    
    /// Longitude is out of valid range (-180 to 180)
    InvalidLongitude(f64),
}

impl fmt::Display for GeoCoordinatesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeoCoordinatesError::InvalidLatitude(lat) => {
                write!(
                    f,
                    "Invalid latitude {}: must be between -90.0 and 90.0",
                    lat
                )
            }
            GeoCoordinatesError::InvalidLongitude(lng) => {
                write!(
                    f,
                    "Invalid longitude {}: must be between -180.0 and 180.0",
                    lng
                )
            }
        }
    }
}

impl std::error::Error for GeoCoordinatesError {}

// ========================================================================
// TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── GeoCoordinates Tests ─────────────────────────────────

    #[test]
    fn test_valid_coordinates() {
        let coords = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        assert_eq!(coords.latitude(), 40.7128);
        assert_eq!(coords.longitude(), -74.0060);
    }

    #[test]
    fn test_coordinates_at_extremes() {
        // North Pole
        let north = GeoCoordinates::new(90.0, 0.0).unwrap();
        assert_eq!(north.latitude(), 90.0);
        
        // South Pole
        let south = GeoCoordinates::new(-90.0, 0.0).unwrap();
        assert_eq!(south.latitude(), -90.0);
        
        // International Date Line
        let dateline_east = GeoCoordinates::new(0.0, 180.0).unwrap();
        assert_eq!(dateline_east.longitude(), 180.0);
        
        let dateline_west = GeoCoordinates::new(0.0, -180.0).unwrap();
        assert_eq!(dateline_west.longitude(), -180.0);
    }

    #[test]
    fn test_invalid_latitude_too_high() {
        let result = GeoCoordinates::new(91.0, 0.0);
        assert!(matches!(result, Err(GeoCoordinatesError::InvalidLatitude(91.0))));
    }

    #[test]
    fn test_invalid_latitude_too_low() {
        let result = GeoCoordinates::new(-91.0, 0.0);
        assert!(matches!(result, Err(GeoCoordinatesError::InvalidLatitude(-91.0))));
    }

    #[test]
    fn test_invalid_longitude_too_high() {
        let result = GeoCoordinates::new(0.0, 181.0);
        assert!(matches!(result, Err(GeoCoordinatesError::InvalidLongitude(181.0))));
    }

    #[test]
    fn test_invalid_longitude_too_low() {
        let result = GeoCoordinates::new(0.0, -181.0);
        assert!(matches!(result, Err(GeoCoordinatesError::InvalidLongitude(-181.0))));
    }

    #[test]
    fn test_nan_coordinates() {
        let result = GeoCoordinates::new(f64::NAN, 0.0);
        assert!(result.is_err());
        
        let result = GeoCoordinates::new(0.0, f64::NAN);
        assert!(result.is_err());
    }

    #[test]
    fn test_infinity_coordinates() {
        let result = GeoCoordinates::new(f64::INFINITY, 0.0);
        assert!(result.is_err());
        
        let result = GeoCoordinates::new(0.0, f64::NEG_INFINITY);
        assert!(result.is_err());
    }

    #[test]
    fn test_coordinates_as_tuple() {
        let coords = GeoCoordinates::new(51.5074, -0.1278).unwrap();
        assert_eq!(coords.as_tuple(), (51.5074, -0.1278));
    }

    #[test]
    fn test_coordinates_display() {
        let coords = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        let display = format!("{}", coords);
        assert!(display.contains("40.7128"));
        assert!(display.contains("-74.0060"));
    }

    // ── Location Tests ────────────────────────────────────────

    #[test]
    fn test_valid_location_with_name() {
        let coords = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        let location = Location::new(
            Some("Home".to_string()),
            "New York".to_string(),
            "United States".to_string(),
            coords,
        ).unwrap();
        
        assert_eq!(location.name(), Some("Home"));
        assert_eq!(location.city(), "New York");
        assert_eq!(location.country(), "United States");
        assert_eq!(location.geoloc().latitude(), 40.7128);
    }

    #[test]
    fn test_valid_location_without_name() {
        let coords = GeoCoordinates::new(51.5074, -0.1278).unwrap();
        let location = Location::new(
            None,
            "London".to_string(),
            "United Kingdom".to_string(),
            coords,
        ).unwrap();
        
        assert_eq!(location.name(), None);
        assert_eq!(location.city(), "London");
    }

    #[test]
    fn test_location_empty_city_error() {
        let coords = GeoCoordinates::new(0.0, 0.0).unwrap();
        let result = Location::new(
            None,
            "".to_string(),
            "Country".to_string(),
            coords,
        );
        assert_eq!(result, Err(LocationError::EmptyCity));
    }

    #[test]
    fn test_location_empty_country_error() {
        let coords = GeoCoordinates::new(0.0, 0.0).unwrap();
        let result = Location::new(
            None,
            "City".to_string(),
            "".to_string(),
            coords,
        );
        assert_eq!(result, Err(LocationError::EmptyCountry));
    }

    #[test]
    fn test_location_empty_name_error() {
        let coords = GeoCoordinates::new(0.0, 0.0).unwrap();
        let result = Location::new(
            Some("   ".to_string()), // Whitespace-only
            "City".to_string(),
            "Country".to_string(),
            coords,
        );
        assert_eq!(result, Err(LocationError::EmptyName));
    }

    #[test]
    fn test_location_trimming() {
        let coords = GeoCoordinates::new(48.8566, 2.3522).unwrap();
        let location = Location::new(
            Some("  Office  ".to_string()),
            "  Paris  ".to_string(),
            "  France  ".to_string(),
            coords,
        ).unwrap();
        
        assert_eq!(location.name(), Some("Office"));
        assert_eq!(location.city(), "Paris");
        assert_eq!(location.country(), "France");
    }

    #[test]
    fn test_location_set_name() {
        let coords = GeoCoordinates::new(35.6762, 139.6503).unwrap();
        let mut location = Location::new(
            None,
            "Tokyo".to_string(),
            "Japan".to_string(),
            coords,
        ).unwrap();
        
        // Set name
        location.set_name(Some("Work".to_string())).unwrap();
        assert_eq!(location.name(), Some("Work"));
        
        // Clear name
        location.set_name(None).unwrap();
        assert_eq!(location.name(), None);
    }

    #[test]
    fn test_location_display_with_name() {
        let coords = GeoCoordinates::new(40.7128, -74.0060).unwrap();
        let location = Location::new(
            Some("Home".to_string()),
            "New York".to_string(),
            "United States".to_string(),
            coords,
        ).unwrap();
        
        let display = format!("{}", location);
        assert_eq!(display, "Home (New York, United States)");
    }

    #[test]
    fn test_location_display_without_name() {
        let coords = GeoCoordinates::new(51.5074, -0.1278).unwrap();
        let location = Location::new(
            None,
            "London".to_string(),
            "United Kingdom".to_string(),
            coords,
        ).unwrap();
        
        let display = format!("{}", location);
        assert_eq!(display, "London, United Kingdom");
    }

    #[test]
    fn test_location_clone_and_eq() {
        let coords = GeoCoordinates::new(48.8566, 2.3522).unwrap();
        let location1 = Location::new(
            Some("Office".to_string()),
            "Paris".to_string(),
            "France".to_string(),
            coords,
        ).unwrap();
        
        let location2 = location1.clone();
        assert_eq!(location1, location2);
    }
}
