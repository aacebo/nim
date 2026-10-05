#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PredictionRow {
    pub id: uuid::Uuid,
    pub version: i32,
    pub hypothesis: String,
    pub confidence: f64,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PredictionMemoryRow {
    pub prediction_id: uuid::Uuid,
    pub memory_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
