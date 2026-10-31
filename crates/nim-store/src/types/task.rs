#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskRow {
    pub id: uuid::Uuid,
    pub parent_id: Option<uuid::Uuid>,
    pub version: i32,
    pub name: String,
    pub source: String,
    pub status: sqlx::types::Json<TaskStatus>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Started {
        started_at: chrono::DateTime<chrono::Utc>,
    },
    Complete {
        started_at: chrono::DateTime<chrono::Utc>,
        ended_at: chrono::DateTime<chrono::Utc>,
    },
    Error {
        reason: String,
        started_at: chrono::DateTime<chrono::Utc>,
        ended_at: chrono::DateTime<chrono::Utc>,
    },
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
