use crate::{Artifact, Chat, Memory, Message};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Record {
    Artifact(Artifact),
    Chat(Chat),
    Message(Message),
    Memory(Memory),
}

impl From<Artifact> for Record {
    fn from(value: Artifact) -> Self {
        Self::Artifact(value)
    }
}

impl From<Chat> for Record {
    fn from(value: Chat) -> Self {
        Self::Chat(value)
    }
}

impl From<Message> for Record {
    fn from(value: Message) -> Self {
        Self::Message(value)
    }
}

impl From<Memory> for Record {
    fn from(value: Memory) -> Self {
        Self::Memory(value)
    }
}

impl TryFrom<nim_store::RecordRow> for Record {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::RecordRow) -> Result<Self, Self::Error> {
        Ok(match value {
            nim_store::RecordRow::Artifact(v) => Self::Artifact(v.try_into()?),
            nim_store::RecordRow::Chat(v) => Self::Chat(v.into()),
            nim_store::RecordRow::Message(v) => Self::Message(v.try_into()?),
            nim_store::RecordRow::Memory(row) => Self::Memory(row.try_into()?),
        })
    }
}
