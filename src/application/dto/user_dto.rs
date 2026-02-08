/// User-related DTOs

use crate::domain::entities::user::Timezone;
use chrono::{Month, NaiveTime, Weekday};

/// Input for registering a new user
#[derive(Debug, Clone)]
pub struct RegisterUserInput {
    pub username: String,
    pub email: String,
    pub password: String, // Plain password - will be hashed by the use case
    pub timezone: Timezone,
}

/// Input for updating user settings
#[derive(Debug, Clone)]
pub struct UpdateUserSettingsInput {
    pub week_start: Option<Weekday>,
    pub year_start: Option<Month>,
    pub day_start: Option<NaiveTime>,
    pub timezone: Option<Timezone>,
}

/// Output after successful registration
#[derive(Debug, Clone)]
pub struct RegisterUserOutput {
    pub user_id: crate::application::types::UserId,
    pub username: String,
}
