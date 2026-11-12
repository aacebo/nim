use crate::{Artifact, Chat, Error, Memory, Message};

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
    type Error = Error;

    fn try_from(value: nim_store::RecordRow) -> Result<Self, Self::Error> {
        Ok(match value {
            nim_store::RecordRow::Artifact(v) => Self::Artifact(v.into()),
            nim_store::RecordRow::Chat(v) => Self::Chat(v.into()),
            nim_store::RecordRow::Message(v) => Self::Message(v.try_into()?),
            nim_store::RecordRow::Memory(row) => Self::Memory(Memory {
                id: row.id.into(),
                version: row.version.into(),
                ty: row.ty.into(),
                contexts: row.contexts.iter().map(|c| c.clone().into()).collect(),
                sources: row.sources.iter().map(|v| (*v).into()).collect(),
                artifacts: Default::default(),
                relations: Default::default(),
                summary: row.summary,
                embedding: row.embedding.map(Into::into),
                salience: row.salience,
                strength: row.strength,
                confidence: row.confidence,
                recalls: row.recalls as u64,
                recalled_at: row.recalled_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }),
        })
    }
}
