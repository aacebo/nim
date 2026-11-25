use crate::{ArtifactAnnotation, MessageAnnotation};

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Span {
    Text { start: u32, end: u32 },
    Image { xmin: f32, ymin: f32, xmax: f32, ymax: f32 },
    Audio { start: f64, end: f64 },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnnotationPartial {
    pub id: AnnotationId,
    pub label: Label,
    pub text: String,
    pub spans: Vec<Span>,
    pub confidence: f32,
}

impl TryFrom<nim_store::ArtifactAnnotationRow> for AnnotationPartial {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::ArtifactAnnotationRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            label: serde_json::from_value(serde_json::Value::String(value.label))?,
            text: value.text,
            spans: serde_json::from_value(value.spans)?,
            confidence: value.confidence,
        })
    }
}

impl From<ArtifactAnnotation> for AnnotationPartial {
    fn from(value: ArtifactAnnotation) -> Self {
        Self {
            id: value.id,
            label: value.label,
            text: value.text,
            spans: value.spans,
            confidence: value.confidence,
        }
    }
}

impl TryFrom<nim_store::MessageAnnotationRow> for AnnotationPartial {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::MessageAnnotationRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            label: serde_json::from_value(serde_json::Value::String(value.label))?,
            text: value.text,
            spans: serde_json::from_value(value.spans)?,
            confidence: value.confidence,
        })
    }
}

impl From<MessageAnnotation> for AnnotationPartial {
    fn from(value: MessageAnnotation) -> Self {
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

impl std::str::FromStr for AnnotationId {
    type Err = nim_error::Diagnostic;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value
            .strip_prefix("annotation_")
            .ok_or_else(|| nim_error::Diagnostic::message("invalid annotation id"))?;

        Ok(Self(uuid::Uuid::parse_str(value)?))
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
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
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
    pub fn try_from_classify_label(label: impl AsRef<str>) -> nim_error::Result<Self> {
        match label.as_ref() {
            "a concrete person, animal, object, or organization" => Ok(Self::Entity),
            "explicit positive or negative sentiment or emotional tone" => Ok(Self::Valence),
            "explicit emotional intensity or physiological arousal" => Ok(Self::Emotion),
            "explicit importance, urgency, or priority for attention" => Ok(Self::Salience),
            "an explicit danger, threat, or risk of harm" => Ok(Self::Threat),
            "an explicit benefit, gain, incentive, or reward" => Ok(Self::Reward),
            "an explicit indication that something is new, unexpected, or surprising" => Ok(Self::Novelty),
            "recognition of something previously encountered or experienced" => Ok(Self::Familiarity),
            "an explicit statement of certainty, uncertainty, or doubt" => Ok(Self::Confidence),
            "the speaker's explicitly stated identity, personal needs, or personal goals" => Ok(Self::SelfRelevance),
            "explicit interpersonal relationships, communication, or social interactions" => Ok(Self::SocialRelevance),
            "dates, times, durations, or the order of events" => Ok(Self::Temporal),
            "physical locations, directions, distances, or spatial relationships" => Ok(Self::Spatial),
            v => Err(nim_error::Diagnostic::message(format!("provided label `{v}` is invalid"))),
        }
    }

    pub fn as_classify_label(&self) -> &'static str {
        match self {
            Self::Entity => "a concrete person, animal, object, or organization",
            Self::Valence => "explicit positive or negative sentiment or emotional tone",
            Self::Emotion => "explicit emotional intensity or physiological arousal",
            Self::Salience => "explicit importance, urgency, or priority for attention",
            Self::Threat => "an explicit danger, threat, or risk of harm",
            Self::Reward => "an explicit benefit, gain, incentive, or reward",
            Self::Novelty => "an explicit indication that something is new, unexpected, or surprising",
            Self::Familiarity => "recognition of something previously encountered or experienced",
            Self::Confidence => "an explicit statement of certainty, uncertainty, or doubt",
            Self::SelfRelevance => "the speaker's explicitly stated identity, personal needs, or personal goals",
            Self::SocialRelevance => "explicit interpersonal relationships, communication, or social interactions",
            Self::Temporal => "dates, times, durations, or the order of events",
            Self::Spatial => "physical locations, directions, distances, or spatial relationships",
            Self::Unknown => "unknown",
        }
    }

    pub fn intensity(&self) -> f32 {
        match self {
            Self::Salience => 1.0,
            Self::Threat => 1.0,
            Self::SelfRelevance => 0.9,
            Self::Novelty => 0.8,
            Self::Emotion => 0.7,
            Self::Reward => 0.6,
            Self::SocialRelevance => 0.5,
            Self::Entity => 0.3,
            Self::Temporal => 0.2,
            Self::Spatial => 0.2,
            Self::Valence => 0.2,
            Self::Familiarity => 0.0,
            Self::Confidence => 0.0,
            Self::Unknown => 0.0,
        }
    }

    pub fn label_multiplier(&self) -> f32 {
        match self {
            Self::Entity => 0.3,
            Self::Valence => 0.2,
            Self::Emotion => 0.7,
            Self::Salience => 1.0,
            Self::Threat => 1.0,
            Self::Reward => 0.6,
            Self::Novelty => 0.8,
            Self::Familiarity => 0.0,
            Self::Confidence => 0.0,
            Self::SelfRelevance => 0.9,
            Self::SocialRelevance => 0.5,
            Self::Temporal => 0.2,
            Self::Spatial => 0.2,
            Self::Unknown => 0.0,
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
