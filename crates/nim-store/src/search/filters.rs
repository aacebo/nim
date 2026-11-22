use crate::TimeRange;

#[derive(Debug, Default)]
pub struct Filter {
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}

#[derive(Debug, Default)]
pub struct ChatFilter {
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}

#[derive(Debug, Default)]
pub struct MemoryFilter {
    pub ty: Option<String>,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
    pub created_at: Option<TimeRange>,
}

#[derive(Debug, Default)]
pub struct MessageFilter {
    pub chat_id: Option<uuid::Uuid>,
    pub role: Option<String>,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}
