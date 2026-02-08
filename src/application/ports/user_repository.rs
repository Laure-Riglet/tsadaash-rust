/// User repository port

use crate::application::errors::AppResult;
use crate::application::types::UserId;
use crate::domain::entities::user::User;

/// Trait for user persistence operations
pub trait UserRepository {
    /// Save a new user
    fn save(&mut self, user: User) -> AppResult<UserId>;
    
    /// Find a user by ID
    fn find_by_id(&self, id: UserId) -> AppResult<User>;
    
    /// Find a user by username
    fn find_by_username(&self, username: &str) -> AppResult<(UserId, User)>;
    
    /// Update an existing user
    fn update(&mut self, id: UserId, user: User) -> AppResult<()>;
    
    /// Check if a username already exists
    fn exists_by_username(&self, username: &str) -> bool;
    
    /// Get the active schedule template ID for a user (if any)
    fn get_active_schedule_template(&self, user_id: UserId) -> AppResult<Option<crate::application::types::ScheduleTemplateId>>;
    
    /// Set the active schedule template for a user
    fn set_active_schedule_template(&mut self, user_id: UserId, template_id: Option<crate::application::types::ScheduleTemplateId>) -> AppResult<()>;
}
