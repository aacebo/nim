use crate::Span;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnnotationRow {
    pub id: uuid::Uuid,
    pub artifact_id: uuid::Uuid,
    pub label: Label,
    pub text: String,
    pub spans: sqlx::types::Json<Vec<Span>>,
    pub confidence: f32,
    pub embedding: Option<pgvector::Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Label {
    Entity,
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
    #[default]
    Unknown,
}
