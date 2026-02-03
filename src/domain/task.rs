#[derive(Debug, Clone)]

pub struct Task {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub is_recurring: bool,
    pub recurrence_interval: Option<String>,
    pub recurrence_unit: Option<String>,
    pub date: Option<String>, // If unique date is needed for non-recurring tasks
    pub created_at: String,
    pub updated_at: String,
}

impl Task {
    pub fn new(
        id: i32,
        user_id: i32,
        title: String,
        is_recurring: bool,
        recurrence_interval: Option<String>,
        recurrence_unit: Option<String>,
        date: Option<String>,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            id,
            user_id,
            title,
            is_recurring,
            recurrence_interval,
            recurrence_unit,
            date,
            created_at,
            updated_at,
        }
    }

    // Getters
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn user_id(&self) -> i32 {
        self.user_id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn is_recurring(&self) -> bool {
        self.is_recurring
    }
    pub fn recurrence_interval(&self) -> Option<&String> {
        self.recurrence_interval.as_ref()
    }
    pub fn recurrence_unit(&self) -> Option<&String> {
        self.recurrence_unit.as_ref()
    }
    pub fn date(&self) -> Option<&String> {
        self.date.as_ref()
    }
    pub fn created_at(&self) -> &str {
        &self.created_at
    }
    pub fn updated_at(&self) -> &str {
        &self.updated_at
    }
}