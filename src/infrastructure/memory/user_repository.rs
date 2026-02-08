/// In-memory user repository implementation

use std::collections::HashMap;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::UserRepository;
use crate::application::types::{UserId, ScheduleTemplateId};
use crate::domain::entities::user::User;

/// In-memory implementation of UserRepository for testing/MVP
pub struct InMemoryUserRepository {
    users: HashMap<UserId, User>,
    username_index: HashMap<String, UserId>,
    active_templates: HashMap<UserId, ScheduleTemplateId>,
    next_id: u64,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            username_index: HashMap::new(),
            active_templates: HashMap::new(),
            next_id: 1,
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn save(&mut self, user: User) -> AppResult<UserId> {
        let id = UserId::new(self.next_id);
        self.next_id += 1;

        self.username_index.insert(user.username.clone(), id);
        self.users.insert(id, user);

        Ok(id)
    }

    fn find_by_id(&self, id: UserId) -> AppResult<User> {
        self.users
            .get(&id)
            .cloned()
            .ok_or(AppError::UserNotFound(id))
    }

    fn find_by_username(&self, username: &str) -> AppResult<(UserId, User)> {
        let id = self.username_index
            .get(username)
            .cloned()
            .ok_or_else(|| AppError::ValidationError(format!("User not found: {}", username)))?;

        let user = self.users
            .get(&id)
            .cloned()
            .ok_or(AppError::UserNotFound(id))?;

        Ok((id, user))
    }

    fn update(&mut self, id: UserId, user: User) -> AppResult<()> {
        if !self.users.contains_key(&id) {
            return Err(AppError::UserNotFound(id));
        }

        // Update username index if username changed
        let old_username = self.users.get(&id).map(|u| u.username.clone());
        if let Some(old) = old_username {
            if old != user.username {
                self.username_index.remove(&old);
                self.username_index.insert(user.username.clone(), id);
            }
        }

        self.users.insert(id, user);
        Ok(())
    }

    fn exists_by_username(&self, username: &str) -> bool {
        self.username_index.contains_key(username)
    }

    fn get_active_schedule_template(&self, user_id: UserId) -> AppResult<Option<ScheduleTemplateId>> {
        if !self.users.contains_key(&user_id) {
            return Err(AppError::UserNotFound(user_id));
        }
        Ok(self.active_templates.get(&user_id).cloned())
    }

    fn set_active_schedule_template(&mut self, user_id: UserId, template_id: Option<ScheduleTemplateId>) -> AppResult<()> {
        if !self.users.contains_key(&user_id) {
            return Err(AppError::UserNotFound(user_id));
        }

        match template_id {
            Some(tid) => {
                self.active_templates.insert(user_id, tid);
            }
            None => {
                self.active_templates.remove(&user_id);
            }
        }

        Ok(())
    }
}
