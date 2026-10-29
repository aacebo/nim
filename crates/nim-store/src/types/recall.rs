#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RecallRow {
    pub id: uuid::Uuid,
    pub trigger: RecallTrigger,
    pub source: RecallSource,
    pub query: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum RecallTrigger {
    Explicit,
    Contextual,
    Associative,
    Sensory,
    Temporal,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum RecallSource {
    User,
    Agent,
    System,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemoryRecallRow {
    pub memory_id: uuid::Uuid,
    pub recall_id: uuid::Uuid,
    pub score: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
