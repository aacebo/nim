use crate::{ArtifactId, Embedding, EntityId, Version};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Chat {
    pub id: ChatId,
    pub version: Version,
    pub name: String,
    pub summary: Option<String>,
    pub confidence: f32,
    pub embedding: Option<Embedding>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChatId(uuid::Uuid);

impl ChatId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for ChatId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<ChatId> for uuid::Uuid {
    fn from(value: ChatId) -> Self {
        value.0
    }
}

impl std::fmt::Display for ChatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "chat_{}", self.0.simple())
    }
}

impl serde::Serialize for ChatId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ChatId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("chat_")
            .ok_or_else(|| serde::de::Error::custom("invalid chat id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

impl From<nim_store::ChatRow> for Chat {
    fn from(value: nim_store::ChatRow) -> Self {
        Self {
            id: value.id.into(),
            version: value.version.into(),
            name: value.name,
            summary: value.summary,
            confidence: value.confidence,
            embedding: value.embedding.map(Into::into),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatEntity {
    pub chat_id: ChatId,
    pub entity_id: EntityId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::ChatEntityRow> for ChatEntity {
    fn from(value: nim_store::ChatEntityRow) -> Self {
        Self {
            chat_id: value.chat_id.into(),
            entity_id: value.entity_id.into(),
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatArtifact {
    pub chat_id: ChatId,
    pub artifact_id: ArtifactId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::ChatArtifactRow> for ChatArtifact {
    fn from(value: nim_store::ChatArtifactRow) -> Self {
        Self {
            chat_id: value.chat_id.into(),
            artifact_id: value.artifact_id.into(),
            created_at: value.created_at,
        }
    }
}
