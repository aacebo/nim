#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnnotationRow {
    pub id: uuid::Uuid,
    pub memory_id: uuid::Uuid,
    pub label: Label,
    pub text: String,
    pub spans: sqlx::types::Json<Vec<Span>>,
    pub confidence: f64,
    pub embedding: Option<pgvector::Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Label {
    Valence,
    Emotion,
    Salience,
    Threat,
    Reward,
    Novelty,
    Familiarity,
    Confidence,
    SelfRelevance,
    SocialRelevance,
    Temporal,
    Spatial,
}
