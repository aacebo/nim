use crate::{MemoryType, Role};

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
    pub ty: Option<MemoryType>,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}

#[derive(Debug, Default)]
pub struct MessageFilter {
    pub chat_id: Option<uuid::Uuid>,
    pub role: Option<Role>,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}
