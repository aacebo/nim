pub use nim_store::Span;

use crate::{Embedding, Error, MemoryId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    pub id: AnnotationId,
    pub artifact_id: MemoryId,
    pub label: Label,
    pub text: String,
    pub spans: Vec<Span>,
    pub confidence: f32,
    #[serde(skip)]
    pub embedding: Option<Embedding>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::AnnotationRow> for Annotation {
    fn from(value: nim_store::AnnotationRow) -> Self {
        Self {
            id: value.id.into(),
            artifact_id: value.artifact_id.into(),
            label: value.label.into(),
            text: value.text,
            spans: value.spans.into_inner(),
            confidence: value.confidence,
            embedding: value.embedding.map(Into::into),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnotationPartial {
    pub id: AnnotationId,
    pub label: Label,
    pub text: String,
    pub spans: Vec<Span>,
    pub confidence: f32,
}

impl From<nim_store::AnnotationRow> for AnnotationPartial {
    fn from(value: nim_store::AnnotationRow) -> Self {
        Self {
            id: value.id.into(),
            label: value.label.into(),
            text: value.text,
            spans: value.spans.into_inner(),
            confidence: value.confidence,
        }
    }
}

impl From<Annotation> for AnnotationPartial {
    fn from(value: Annotation) -> Self {
        Self {
            id: value.id,
            label: value.label,
            text: value.text,
            spans: value.spans,
            confidence: value.confidence,
        }
    }
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

#[non_exhaustive]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Label {
    /// person/place/thing
    Entity,

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

    /// unknown
    #[default]
    Unknown,
}

impl Label {
    pub fn try_from_classify_label(label: impl AsRef<str>) -> Result<Self, Error> {
        match label.as_ref() {
            "a person, animal, object, or organization" => Ok(Self::Entity),
            "positive or negative feelings" => Ok(Self::Valence),
            "an emotion or emotional reaction" => Ok(Self::Emotion),
            "importance or attention priority" => Ok(Self::Salience),
            "danger, harm, or risk" => Ok(Self::Threat),
            "a benefit, gain, or reward" => Ok(Self::Reward),
            "something new or unexpected" => Ok(Self::Novelty),
            "something known or familiar" => Ok(Self::Familiarity),
            "certainty, doubt, or uncertainty" => Ok(Self::Confidence),
            "the speaker's identity, needs, or goals" => Ok(Self::SelfRelevance),
            "relationships or social interactions" => Ok(Self::SocialRelevance),
            "a time, date, or duration" => Ok(Self::Temporal),
            "a location, direction, or spacial relationship" => Ok(Self::Spatial),
            v => Err(Error::custom(format!("provided label `{v}` is invalid"))),
        }
    }

    pub fn as_classify_label(&self) -> &'static str {
        match self {
            Self::Entity => "a person, animal, object, or organization",
            Self::Valence => "positive or negative feelings",
            Self::Emotion => "an emotion or emotional reaction",
            Self::Salience => "importance or attention priority",
            Self::Threat => "danger, harm, or risk",
            Self::Reward => "a benefit, gain, or reward",
            Self::Novelty => "something new or unexpected",
            Self::Familiarity => "something known or familiar",
            Self::Confidence => "certainty, doubt, or uncertainty",
            Self::SelfRelevance => "the speaker's identity, needs, or goals",
            Self::SocialRelevance => "relationships or social interactions",
            Self::Temporal => "a time, date, or duration",
            Self::Spatial => "a location, direction, or spacial relationship",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entity => write!(f, "entity"),
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
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<nim_store::Label> for Label {
    fn from(value: nim_store::Label) -> Self {
        match value {
            nim_store::Label::Entity => Self::Entity,
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
            nim_store::Label::Unknown => Self::Unknown,
            _ => unreachable!(),
        }
    }
}

impl From<Label> for nim_store::Label {
    fn from(value: Label) -> Self {
        match value {
            Label::Entity => Self::Entity,
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
            Label::Unknown => Self::Unknown,
        }
    }
}
