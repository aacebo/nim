#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RecallRow {
    pub id: uuid::Uuid,
    pub trigger: String,
    pub source: String,
    pub query: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemoryRecallRow {
    pub memory_id: uuid::Uuid,
    pub recall_id: uuid::Uuid,
    pub score: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
