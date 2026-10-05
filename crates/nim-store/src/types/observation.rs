#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ObservationRow {
    pub id: uuid::Uuid,
    pub source: String,
    pub description: String,
    pub embedding: Option<pgvector::Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ObservationMemoryRow {
    pub observation_id: uuid::Uuid,
    pub memory_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
