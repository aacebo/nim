#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct MemoryRow {
    pub id: uuid::Uuid,
    pub version: i32,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub ty: String,
    pub sources: sqlx::types::JsonValue,
    pub contexts: sqlx::types::JsonValue,
    pub artifacts: sqlx::types::JsonValue,
    pub summary: Option<String>,
    #[serde(skip)]
    pub embedding: Option<pgvector::Vector>,
    pub salience: f32,
    pub strength: f32,
    pub confidence: f32,
    pub recalls: i64,
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
