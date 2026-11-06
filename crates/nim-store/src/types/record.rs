use crate::{ArtifactRow, ChatRow, MemoryRow, MessageRow};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum RecordRow {
    Artifact(ArtifactRow),
    Chat(ChatRow),
    Message(MessageRow),
    Memory(MemoryRow),
}

impl From<ArtifactRow> for RecordRow {
    fn from(value: ArtifactRow) -> Self {
        Self::Artifact(value)
    }
}

impl From<ChatRow> for RecordRow {
    fn from(value: ChatRow) -> Self {
        Self::Chat(value)
    }
}

impl From<MessageRow> for RecordRow {
    fn from(value: MessageRow) -> Self {
        Self::Message(value)
    }
}

impl From<MemoryRow> for RecordRow {
    fn from(value: MemoryRow) -> Self {
        Self::Memory(value)
    }
}

impl<'de> serde::Deserialize<'de> for RecordRow {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if value.get("chat_id").is_some() {
            serde_json::from_value(value).map(Self::Message)
        } else if value.get("content").is_some() {
            serde_json::from_value(value).map(Self::Artifact)
        } else if value.get("sources").is_some() {
            serde_json::from_value(value).map(Self::Memory)
        } else if value.get("last_analyzed_id").is_some() {
            serde_json::from_value(value).map(Self::Chat)
        } else {
            return Err(serde::de::Error::custom("unrecognized record shape"));
        }
        .map_err(serde::de::Error::custom)
    }
}
