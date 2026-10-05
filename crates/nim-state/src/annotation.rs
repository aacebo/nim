use crate::Embedding;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    pub id: AnnotationId,
    pub label: Label,
    pub text: String,
    pub spans: Vec<Span>,
    pub confidence: f32,
    pub embedding: Option<Embedding>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnnotationId(uuid::Uuid);

impl std::fmt::Display for AnnotationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "annotation_{}", self.0)
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
