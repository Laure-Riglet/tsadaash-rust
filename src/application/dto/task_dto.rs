/// Task-related DTOs

use crate::domain::entities::task::{Periodicity, TaskPriority};
use crate::domain::entities::user::Location;
use crate::domain::entities::schedule::{AvailabilityLevel, DeviceAccess, Mobility};

/// Input for creating a new task
#[derive(Debug, Clone)]
pub struct CreateTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub periodicity: Periodicity,
    
    // Scheduling attributes
    pub min_hands: Option<AvailabilityLevel>,
    pub min_eyes: Option<AvailabilityLevel>,
    pub min_speech: Option<AvailabilityLevel>,
    pub min_cognitive: Option<AvailabilityLevel>,
    pub min_device: Option<DeviceAccess>,
    pub allowed_mobility: Option<Mobility>,
    pub locations: Vec<Option<Location>>,
}

/// Input for updating an existing task
#[derive(Debug, Clone)]
pub struct UpdateTaskInput {
    pub title: Option<String>,
    pub description: Option<Option<String>>, // Option<Option<>> allows clearing the description
    pub priority: Option<TaskPriority>,
    pub periodicity: Option<Periodicity>,
    
    // Scheduling attributes
    pub min_hands: Option<AvailabilityLevel>,
    pub min_eyes: Option<AvailabilityLevel>,
    pub min_speech: Option<AvailabilityLevel>,
    pub min_cognitive: Option<AvailabilityLevel>,
    pub min_device: Option<DeviceAccess>,
    pub allowed_mobility: Option<Mobility>,
    pub locations: Option<Vec<Option<Location>>>,
}

/// Input for completing an occurrence rep
#[derive(Debug, Clone)]
pub struct CompleteOccurrenceRepInput {
    pub task_id: crate::application::types::TaskId,
    pub occurrence_index: usize,
    pub rep_index: usize,
    pub notes: Option<String>,
}

/// Output after task creation
#[derive(Debug, Clone)]
pub struct CreateTaskOutput {
    pub task_id: crate::application::types::TaskId,
    pub title: String,
}
