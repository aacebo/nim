#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FactRow {
    pub id: uuid::Uuid,
    pub description: String,
    pub confidence: f32,
    pub embedding: Option<pgvector::Vector>,
    pub recalls: i64,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FactMemoryRow {
    pub fact_id: uuid::Uuid,
    pub memory_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
