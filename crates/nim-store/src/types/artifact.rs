#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ArtifactRow {
    pub id: uuid::Uuid,
    pub version: i32,
    pub name: String,
    pub summary: Option<String>,
    #[serde(skip)]
    pub embedding: Option<pgvector::Vector>,
    pub confidence: f32,
    pub salience: f32,
    pub strength: f32,
    pub content: sqlx::types::JsonValue,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemoryArtifactRow {
    pub memory_id: uuid::Uuid,
    pub artifact_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageArtifactRow {
    pub message_id: uuid::Uuid,
    pub artifact_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ArtifactAnnotationRow {
    pub id: uuid::Uuid,
    pub version: i32,
    pub artifact_id: uuid::Uuid,
    pub label: String,
    pub text: String,
    pub spans: sqlx::types::JsonValue,
    pub confidence: f32,
    pub embedding: Option<pgvector::Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
