#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct MessageRow {
    pub id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
    pub reply_to_id: Option<uuid::Uuid>,
    pub version: i32,
    pub role: String,
    pub content: sqlx::types::JsonValue,
    pub artifacts: sqlx::types::JsonValue,
    pub summary: Option<String>,
    #[serde(skip)]
    pub embedding: Option<pgvector::Vector>,
    pub confidence: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageAnnotationRow {
    pub id: uuid::Uuid,
    pub version: i32,
    pub message_id: uuid::Uuid,
    pub label: String,
    pub text: String,
    pub spans: sqlx::types::JsonValue,
    pub confidence: f32,
    pub embedding: Option<pgvector::Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
