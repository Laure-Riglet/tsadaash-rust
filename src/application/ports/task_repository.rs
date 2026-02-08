/// Task repository port

use crate::application::errors::AppResult;
use crate::application::types::{TaskId, UserId};
use crate::domain::entities::task::Task;
use chrono::{DateTime, Utc};

/// Trait for task persistence operations
pub trait TaskRepository {
    /// Save a new task for a user
    fn save(&mut self, user_id: UserId, task: Task) -> AppResult<TaskId>;
    
    /// Find a task by ID (and verify it belongs to the user)
    fn find_by_id(&self, user_id: UserId, task_id: TaskId) -> AppResult<Task>;
    
    /// Update an existing task
    fn update(&mut self, user_id: UserId, task_id: TaskId, task: Task) -> AppResult<()>;
    
    /// Delete a task
    fn delete(&mut self, user_id: UserId, task_id: TaskId) -> AppResult<()>;
    
    /// List all tasks for a user
    fn list_by_user(&self, user_id: UserId) -> AppResult<Vec<(TaskId, Task)>>;
    
    /// List active tasks for a user
    fn list_active_by_user(&self, user_id: UserId) -> AppResult<Vec<(TaskId, Task)>>;
    
    /// Find tasks that should occur on a specific date
    fn find_tasks_for_date(&self, user_id: UserId, date: DateTime<Utc>) -> AppResult<Vec<(TaskId, Task)>>;
}
