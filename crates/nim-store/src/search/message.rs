use crate::Role;

#[derive(Debug, Default)]
pub struct MessageFilter {
    pub chat_id: Option<uuid::Uuid>,
    pub role: Option<Role>,
    pub limit: Option<usize>,
    pub min_score: Option<f32>,
}
