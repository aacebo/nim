use crate::{Embedding, MemoryId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    pub id: AnnotationId,
    pub memory_id: MemoryId,
    pub label: Label,
    pub text: String,
    pub spans: Vec<Span>,
    pub confidence: f32,
    pub embedding: Option<Embedding>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnnotationId(uuid::Uuid);

impl AnnotationId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for AnnotationId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<AnnotationId> for uuid::Uuid {
    fn from(value: AnnotationId) -> Self {
        value.0
    }
}

impl std::fmt::Display for AnnotationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "annotation_{}", self.0.simple())
    }
}

impl serde::Serialize for AnnotationId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for AnnotationId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("annotation_")
            .ok_or_else(|| serde::de::Error::custom("invalid annotation id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Label {
    /// positive ↔ negative affect
    Valence,

    /// physiological/emotional intensity
    Emotion,

    /// importance / attention priority
    Salience,

    /// danger relevance
    Threat,

    /// expected or experienced value
    Reward,

    /// how unexpected/new something is
    Novelty,

    /// recognition strength
    Familiarity,

    /// certainty of representation/retrieval
    Confidence,

    /// relation to current goals/self-model
    SelfRelevance,

    /// people/social context
    SocialRelevance,

    /// when
    Temporal,

    /// where
    Spatial,
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valence => write!(f, "valence"),
            Self::Emotion => write!(f, "emotion"),
            Self::Salience => write!(f, "salience"),
            Self::Threat => write!(f, "threat"),
            Self::Reward => write!(f, "reward"),
            Self::Novelty => write!(f, "novelty"),
            Self::Familiarity => write!(f, "familiarity"),
            Self::Confidence => write!(f, "confidence"),
            Self::SelfRelevance => write!(f, "self_relevance"),
            Self::SocialRelevance => write!(f, "social_relevance"),
            Self::Temporal => write!(f, "temporal"),
            Self::Spatial => write!(f, "spatial"),
        }
    }
}

impl From<nim_store::AnnotationRow> for Annotation {
    fn from(value: nim_store::AnnotationRow) -> Self {
        Self {
            id: value.id.into(),
            memory_id: value.memory_id.into(),
            label: value.label.into(),
            text: value.text,
            spans: value.spans.into_inner().into_iter().map(Into::into).collect(),
            confidence: value.confidence as f32,
            embedding: value.embedding.map(Into::into),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<nim_store::Label> for Label {
    fn from(value: nim_store::Label) -> Self {
        match value {
            nim_store::Label::Valence => Self::Valence,
            nim_store::Label::Emotion => Self::Emotion,
            nim_store::Label::Salience => Self::Salience,
            nim_store::Label::Threat => Self::Threat,
            nim_store::Label::Reward => Self::Reward,
            nim_store::Label::Novelty => Self::Novelty,
            nim_store::Label::Familiarity => Self::Familiarity,
            nim_store::Label::Confidence => Self::Confidence,
            nim_store::Label::SelfRelevance => Self::SelfRelevance,
            nim_store::Label::SocialRelevance => Self::SocialRelevance,
            nim_store::Label::Temporal => Self::Temporal,
            nim_store::Label::Spatial => Self::Spatial,
            _ => unreachable!(),
        }
    }
}

impl From<Label> for nim_store::Label {
    fn from(value: Label) -> Self {
        match value {
            Label::Valence => Self::Valence,
            Label::Emotion => Self::Emotion,
            Label::Salience => Self::Salience,
            Label::Threat => Self::Threat,
            Label::Reward => Self::Reward,
            Label::Novelty => Self::Novelty,
            Label::Familiarity => Self::Familiarity,
            Label::Confidence => Self::Confidence,
            Label::SelfRelevance => Self::SelfRelevance,
            Label::SocialRelevance => Self::SocialRelevance,
            Label::Temporal => Self::Temporal,
            Label::Spatial => Self::Spatial,
        }
    }
}

impl From<nim_store::Span> for Span {
    fn from(value: nim_store::Span) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<Span> for nim_store::Span {
    fn from(value: Span) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}
