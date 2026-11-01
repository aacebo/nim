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
