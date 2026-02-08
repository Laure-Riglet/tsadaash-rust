//! # Application Configuration
//! 
//! Centralized configuration for default values and application settings.
//! Values can be overridden via environment variables (loaded from .env file).
//! 
//! ## Environment Variables
//! 
//! Create a `.env` file in the project root with any of these variables:
//! 
//! ```text
//! # Task Settings
//! TASK_MAX_TITLE_LENGTH=200
//! TASK_MAX_DESCRIPTION_LENGTH=2000
//! TASK_DEFAULT_DURATION_MINUTES=30
//! 
//! # Task Occurrence Settings
//! OCCURRENCE_MAX_NOTES_LENGTH=1000
//! OCCURRENCE_REP_MAX_NOTES_LENGTH=500
//! 
//! # Schedule Settings (BusyButFlexible constraints)
//! SCHEDULE_BUSY_FLEX_MAX_MINUTES=15
//! SCHEDULE_BUSY_FLEX_MAX_HANDS_LEVEL=1  # 0=None, 1=Limited, 2=Full
//! SCHEDULE_BUSY_FLEX_MAX_EYES_LEVEL=1
//! SCHEDULE_BUSY_FLEX_MAX_DEVICE_LEVEL=1  # 0=None, 1=PhoneOnly, 2=Computer
//! ```

use once_cell::sync::Lazy;
use std::env;

// ========================================================================
// CONFIGURATION STRUCT
// ========================================================================

/// Global application configuration
pub struct Config {
    // ── TASK SETTINGS ───────────────────────────────────────
    pub task_max_title_length: usize,
    pub task_max_description_length: usize,
    pub task_default_duration_minutes: u16,
    
    // ── TASK OCCURRENCE SETTINGS ────────────────────────────
    pub occurrence_max_notes_length: usize,
    pub occurrence_rep_max_notes_length: usize,
    
    // ── SCHEDULE SETTINGS ───────────────────────────────────
    pub schedule_busy_flex_max_minutes: u32,
    pub schedule_busy_flex_max_hands_level: u8,
    pub schedule_busy_flex_max_eyes_level: u8,
    pub schedule_busy_flex_max_device_level: u8,
}

impl Config {
    /// Load configuration from environment variables (with defaults)
    fn load() -> Self {
        // Try to load .env file (optional, fails silently if not found)
        let _ = dotenv::dotenv();

        Self {
            // Task settings
            task_max_title_length: env_var_or("TASK_MAX_TITLE_LENGTH", 200),
            task_max_description_length: env_var_or("TASK_MAX_DESCRIPTION_LENGTH", 2000),
            task_default_duration_minutes: env_var_or("TASK_DEFAULT_DURATION_MINUTES", 30),
            
            // Task occurrence settings
            occurrence_max_notes_length: env_var_or("OCCURRENCE_MAX_NOTES_LENGTH", 1000),
            occurrence_rep_max_notes_length: env_var_or("OCCURRENCE_REP_MAX_NOTES_LENGTH", 500),
            
            // Schedule settings
            schedule_busy_flex_max_minutes: env_var_or("SCHEDULE_BUSY_FLEX_MAX_MINUTES", 15),
            schedule_busy_flex_max_hands_level: env_var_or("SCHEDULE_BUSY_FLEX_MAX_HANDS_LEVEL", 1),
            schedule_busy_flex_max_eyes_level: env_var_or("SCHEDULE_BUSY_FLEX_MAX_EYES_LEVEL", 1),
            schedule_busy_flex_max_device_level: env_var_or("SCHEDULE_BUSY_FLEX_MAX_DEVICE_LEVEL", 1),
        }
    }
}

/// Parse environment variable or return default value
fn env_var_or<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    env::var(key)
        .ok()
        .and_then(|s| s.parse::<T>().ok())
        .unwrap_or(default)
}

// ========================================================================
// GLOBAL CONFIG INSTANCE
// ========================================================================

/// Global configuration instance (lazy-loaded on first access)
pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);

// ========================================================================
// CONVENIENCE ACCESSORS
// ========================================================================

// Task
pub fn task_max_title_length() -> usize {
    CONFIG.task_max_title_length
}

pub fn task_max_description_length() -> usize {
    CONFIG.task_max_description_length
}

pub fn task_default_duration_minutes() -> u16 {
    CONFIG.task_default_duration_minutes
}

// Task Occurrence
pub fn occurrence_max_notes_length() -> usize {
    CONFIG.occurrence_max_notes_length
}

pub fn occurrence_rep_max_notes_length() -> usize {
    CONFIG.occurrence_rep_max_notes_length
}

// Schedule
pub fn schedule_busy_flex_max_minutes() -> u32 {
    CONFIG.schedule_busy_flex_max_minutes
}

pub fn schedule_busy_flex_max_hands_level() -> u8 {
    CONFIG.schedule_busy_flex_max_hands_level
}

pub fn schedule_busy_flex_max_eyes_level() -> u8 {
    CONFIG.schedule_busy_flex_max_eyes_level
}

pub fn schedule_busy_flex_max_device_level() -> u8 {
    CONFIG.schedule_busy_flex_max_device_level
}

// ========================================================================
// TESTS
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loads_defaults() {
        // Config should load without errors
        let config = Config::load();
        
        assert_eq!(config.task_max_title_length, 200);
        assert_eq!(config.task_max_description_length, 2000);
        assert_eq!(config.task_default_duration_minutes, 30);
        assert_eq!(config.occurrence_max_notes_length, 1000);
        assert_eq!(config.occurrence_rep_max_notes_length, 500);
        assert_eq!(config.schedule_busy_flex_max_minutes, 15);
    }

    #[test]
    fn test_accessor_functions() {
        assert!(task_max_title_length() > 0);
        assert!(task_max_description_length() > 0);
        assert!(task_default_duration_minutes() > 0);
        assert!(occurrence_max_notes_length() > 0);
        assert!(schedule_busy_flex_max_minutes() > 0);
    }
}
