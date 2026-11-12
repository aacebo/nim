pub use nim_store::Role;

use crate::{AnnotationId, ArtifactId, ChatId, Embedding, Label, Span, Version};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub chat_id: ChatId,
    pub parent_id: Option<MessageId>,
    pub version: Version,
    pub role: Role,
    pub content: MessageContent,
    pub summary: Option<String>,
    #[serde(skip)]
    pub embedding: Option<Embedding>,
    pub confidence: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageId(uuid::Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for MessageId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<MessageId> for uuid::Uuid {
    fn from(value: MessageId) -> Self {
        value.0
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "message_{}", self.0.simple())
    }
}

impl serde::Serialize for MessageId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for MessageId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("message_")
            .ok_or_else(|| serde::de::Error::custom("invalid message id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

impl TryFrom<nim_store::MessageRow> for Message {
    type Error = crate::Error;

    fn try_from(value: nim_store::MessageRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            chat_id: value.chat_id.into(),
            parent_id: value.parent_id.map(Into::into),
            version: value.version.into(),
            role: value.role,
            content: serde_json::from_value(value.content)?,
            summary: value.summary,
            embedding: value.embedding.map(Into::into),
            confidence: value.confidence,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
#[serde(untagged)]
pub enum MessageContent {
    Text {
        #[validate(min_length = 1)]
        text: String,
    },
    Url {
        #[validate(custom = validate::url)]
        url: String,
    },
    Base64 {
        #[validate(min_length = 1)]
        base64: String,
    },
    Artifact {
        artifact_id: ArtifactId,
    },
    #[validate(min_items = 1)]
    Array(#[validate] Vec<Self>),
    #[validate]
    String(#[validate(min_length = 1)] String),
}

impl MessageContent {
    pub fn artifacts(&self) -> Vec<ArtifactId> {
        let mut items = Vec::new();

        match self {
            Self::Artifact { artifact_id } => items.push(*artifact_id),
            Self::Array(content) => items.extend(content.iter().flat_map(|v| v.artifacts())),
            _ => {}
        };

        items
    }
}

impl std::fmt::Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(v) => write!(f, "{v}"),
            Self::Text { text } => write!(f, "{text}"),
            Self::Url { url } => write!(f, "{url}"),
            Self::Base64 { base64 } => write!(f, "{base64}"),
            Self::Artifact { artifact_id } => write!(f, "{artifact_id}"),
            Self::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    if i < items.len() - 1 {
                        writeln!(f, "{item}")?;
                    } else {
                        write!(f, "{item}")?;
                    }
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageAnnotation {
    pub id: AnnotationId,
    pub version: Version,
    pub message_id: MessageId,
    pub label: Label,
    pub text: String,
    pub spans: Vec<Span>,
    pub confidence: f32,
    #[serde(skip)]
    pub embedding: Option<Embedding>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::MessageAnnotationRow> for MessageAnnotation {
    fn from(value: nim_store::MessageAnnotationRow) -> Self {
        Self {
            id: value.id.into(),
            version: value.version.into(),
            message_id: value.message_id.into(),
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

mod validate {
    use serde_valid::validation::Error;

    pub fn url(value: &str) -> Result<(), Error> {
        url::Url::parse(value)
            .map(|_| ())
            .map_err(|_| Error::Custom("invalid url".into()))
    }
}
