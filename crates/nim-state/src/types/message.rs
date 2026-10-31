pub use nim_store::{FileContent, MessageContent, Role, TextContent};

use crate::{ChatId, Embedding, Version};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub chat_id: ChatId,
    pub version: Version,
    pub role: Role,
    pub content: Vec<MessageContent>,
    pub summary: Option<String>,
    pub confidence: f32,
    pub embedding: Option<Embedding>,
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

impl From<nim_store::MessageRow> for Message {
    fn from(value: nim_store::MessageRow) -> Self {
        Self {
            id: value.id.into(),
            chat_id: value.chat_id.into(),
            version: value.version.into(),
            role: value.role,
            content: value.content.into_inner(),
            summary: value.summary,
            confidence: value.confidence,
            embedding: value.embedding.map(Into::into),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
