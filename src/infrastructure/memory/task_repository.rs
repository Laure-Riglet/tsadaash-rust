/// In-memory task repository implementation

use std::collections::HashMap;
use crate::application::errors::{AppError, AppResult};
use crate::application::ports::TaskRepository;
use crate::application::types::{TaskId, UserId};
use crate::domain::entities::task::Task;
use chrono::{DateTime, Utc};

/// Key for storing tasks per user
type TaskKey = (UserId, TaskId);

/// In-memory implementation of TaskRepository for testing/MVP
pub struct InMemoryTaskRepository {
    tasks: HashMap<TaskKey, Task>,
    next_id: u64,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
        }
    }
}

impl TaskRepository for InMemoryTaskRepository {
    fn save(&mut self, user_id: UserId, task: Task) -> AppResult<TaskId> {
        let task_id = TaskId::new(self.next_id);
        self.next_id += 1;

        self.tasks.insert((user_id, task_id), task);

        Ok(task_id)
    }

    fn find_by_id(&self, user_id: UserId, task_id: TaskId) -> AppResult<Task> {
        self.tasks
            .get(&(user_id, task_id))
            .cloned()
            .ok_or(AppError::TaskNotFound(task_id))
    }

    fn update(&mut self, user_id: UserId, task_id: TaskId, task: Task) -> AppResult<()> {
        let key = (user_id, task_id);
        if !self.tasks.contains_key(&key) {
            return Err(AppError::TaskNotFound(task_id));
        }

        self.tasks.insert(key, task);
        Ok(())
    }

    fn delete(&mut self, user_id: UserId, task_id: TaskId) -> AppResult<()> {
        let key = (user_id, task_id);
        self.tasks.remove(&key)
            .ok_or(AppError::TaskNotFound(task_id))?;
        Ok(())
    }

    fn list_by_user(&self, user_id: UserId) -> AppResult<Vec<(TaskId, Task)>> {
        let tasks: Vec<(TaskId, Task)> = self.tasks
            .iter()
            .filter(|((uid, _), _)| *uid == user_id)
            .map(|((_, tid), task)| (*tid, task.clone()))
            .collect();

        Ok(tasks)
    }

    fn list_active_by_user(&self, user_id: UserId) -> AppResult<Vec<(TaskId, Task)>> {
        let tasks: Vec<(TaskId, Task)> = self.tasks
            .iter()
            .filter(|((uid, _), task)| *uid == user_id && task.is_active())
            .map(|((_, tid), task)| (*tid, task.clone()))
            .collect();

        Ok(tasks)
    }

    fn find_tasks_for_date(&self, user_id: UserId, date: DateTime<Utc>) -> AppResult<Vec<(TaskId, Task)>> {
        // For MVP, return all active tasks
        // In a real implementation, you'd check periodicity to see if the task should occur on this date
        // Note: We use Monday as default week_start since we don't have user context here
        // In a full implementation, this would need to be passed in or fetched
        use chrono::Weekday;
        let week_start = Weekday::Mon;
        
        let tasks: Vec<(TaskId, Task)> = self.tasks
            .iter()
            .filter(|((uid, _), task)| {
                *uid == user_id && task.is_active() && task.should_occur_on(&date, week_start)
            })
            .map(|((_, tid), task)| (*tid, task.clone()))
            .collect();

        Ok(tasks)
    }
}
