#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModelRow {
    pub id: uuid::Uuid,
    pub version: i32,
    #[sqlx(rename = "type")]
    pub ty: String,
    pub name: String,
    pub tasks: Vec<String>,
    pub architectures: Vec<String>,
    pub source: sqlx::types::JsonValue,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModelWeightRow {
    pub id: uuid::Uuid,
    pub model_id: uuid::Uuid,
    pub version: i32,
    pub name: String,
    pub content: sqlx::types::JsonValue,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
