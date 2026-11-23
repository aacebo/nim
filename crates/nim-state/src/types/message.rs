use crate::{AnnotationId, ArtifactId, ChatId, Embedding, Label, Span, Version};

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
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
#[serde(untagged)]
pub enum MessageContent {
    #[validate]
    String(#[validate(min_length = 1)] String),

    #[validate(min_items = 1)]
    Parts(#[validate] Vec<MessageContentPart>),
}

impl MessageContent {
    pub fn artifacts(&self) -> Vec<ArtifactId> {
        match self {
            Self::String(_) => Default::default(),
            Self::Parts(arr) => arr.iter().flat_map(|v| v.artifacts()).collect(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContentPart {
    #[validate]
    Text {
        #[validate(min_length = 1)]
        text: String,
    },
    Artifact {
        artifact_id: ArtifactId,
    },
    Json {
        json: serde_json::Value,
    },
    Url {
        url: url::Url,
    },
}

impl MessageContentPart {
    pub fn artifacts(&self) -> Vec<ArtifactId> {
        match self {
            Self::Artifact { artifact_id } => vec![*artifact_id],
            _ => Default::default(),
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
