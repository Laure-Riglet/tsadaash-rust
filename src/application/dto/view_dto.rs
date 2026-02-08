/// View/Query DTOs

use crate::application::types::TaskId;
use chrono::{DateTime, FixedOffset};
use crate::domain::entities::schedule::TimeBlock;

/// Input for getting a day overview
#[derive(Debug, Clone)]
pub struct GetDayOverviewInput {
    pub date: DateTime<FixedOffset>,
}

/// A time slot with a task scheduled in it
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub task_id: TaskId,
    pub title: String,
    pub time_block: TimeBlock,
    pub occurrence_index: usize,
}

/// A suggested time slot where a task could be scheduled
#[derive(Debug, Clone)]
pub struct SuggestedSlot {
    pub time_block: TimeBlock,
    pub score: u8, // 0-100, higher is better
    pub reason: String,
}

/// Output for day overview query
#[derive(Debug, Clone)]
pub struct DayOverview {
    pub date: DateTime<FixedOffset>,
    pub time_blocks: Vec<TimeBlock>,
    pub scheduled_tasks: Vec<ScheduledTask>,
    pub suggestions: Vec<(TaskId, Vec<SuggestedSlot>)>, // Task ID -> suggested slots
}
