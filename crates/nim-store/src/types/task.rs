use crate::Source;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskRow {
    pub id: uuid::Uuid,
    pub parent_id: Option<uuid::Uuid>,
    pub version: i32,
    pub name: String,
    pub source: sqlx::types::Json<Source>,
    pub status: sqlx::types::JsonValue,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModelTaskRow {
    pub model_id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ArtifactTaskRow {
    pub artifact_id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChatTaskRow {
    pub chat_id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageTaskRow {
    pub message_id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskEventRow {
    pub id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub content: sqlx::types::JsonValue,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
