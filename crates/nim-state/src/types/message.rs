use crate::{AnnotationId, ArtifactPartial, ChatId, Embedding, Label, Span, Version};
use nim_chat::ChatMessageContent;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    Agent,
    User,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Agent => write!(f, "agent"),
            Self::User => write!(f, "user"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub chat_id: ChatId,
    pub reply_to_id: Option<MessageId>,
    pub version: Version,
    pub role: Role,
    pub content: ChatMessageContent,
    pub summary: Option<String>,
    #[serde(skip)]
    pub embedding: Option<Embedding>,
    pub confidence: f32,
    pub artifacts: Vec<ArtifactPartial>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessagePartial {
    pub id: MessageId,
    pub role: Role,
    pub content: ChatMessageContent,
    pub artifacts: Vec<ArtifactPartial>,
    pub summary: Option<String>,
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

impl std::str::FromStr for MessageId {
    type Err = nim_error::Diagnostic;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value
            .strip_prefix("message_")
            .ok_or_else(|| nim_error::Diagnostic::message("invalid message id"))?;

        Ok(Self(uuid::Uuid::parse_str(value)?))
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
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

impl TryFrom<nim_store::MessageRow> for Message {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::MessageRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            chat_id: value.chat_id.into(),
            reply_to_id: value.reply_to_id.map(Into::into),
            version: value.version.into(),
            role: serde_json::from_value(serde_json::Value::String(value.role))?,
            content: serde_json::from_value(value.content)?,
            summary: value.summary,
            embedding: value.embedding.map(Into::into),
            confidence: value.confidence,
            artifacts: serde_json::from_value(value.artifacts)?,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
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

impl TryFrom<nim_store::MessageAnnotationRow> for MessageAnnotation {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::MessageAnnotationRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            version: value.version.into(),
            message_id: value.message_id.into(),
            label: serde_json::from_value(serde_json::Value::String(value.label))?,
            text: value.text,
            spans: serde_json::from_value(value.spans)?,
            confidence: value.confidence,
            embedding: value.embedding.map(Into::into),
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}
