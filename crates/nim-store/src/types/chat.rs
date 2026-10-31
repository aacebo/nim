#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChatRow {
    pub id: uuid::Uuid,
    pub version: i32,
    pub summary: Option<String>,
    pub confidence: f32,
    pub embedding: Option<pgvector::Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChatEntityRow {
    pub chat_id: uuid::Uuid,
    pub entity_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChatArtifactRow {
    pub chat_id: uuid::Uuid,
    pub artifact_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
