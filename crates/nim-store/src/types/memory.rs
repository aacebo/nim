use super::Context;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemoryRow {
    pub id: uuid::Uuid,
    pub version: i32,
    #[sqlx(rename = "type")]
    pub ty: MemoryType,
    pub contexts: sqlx::types::Json<Vec<Context>>,
    pub salience: f64,
    pub strength: f64,
    pub confidence: f64,
    pub recalls: i64,
    pub description: String,
    pub summary: Option<String>,
    pub embedding: Option<pgvector::Vector>,
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemoryRelationRow {
    pub source_id: uuid::Uuid,
    pub target_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum MemoryType {
    Episodic,
    Semantic,
    Procedural,
    Working,
}
