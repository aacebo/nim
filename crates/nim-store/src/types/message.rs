#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageRow {
    pub id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
    pub parent_id: Option<uuid::Uuid>,
    pub version: i32,
    pub role: Role,
    pub content: sqlx::types::JsonValue,
    pub summary: Option<String>,
    pub confidence: f32,
    pub embedding: Option<pgvector::Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Role {
    System,
    Agent,
    User,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageAnnotationRow {
    pub id: uuid::Uuid,
    pub version: i32,
    pub message_id: uuid::Uuid,
    pub label: crate::Label,
    pub text: String,
    pub spans: sqlx::types::Json<Vec<crate::Span>>,
    pub confidence: f32,
    pub embedding: Option<pgvector::Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
