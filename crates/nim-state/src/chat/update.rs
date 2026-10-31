use crate::{Chat, ChatId, ChatService, Error};

#[derive(Debug, Default)]
pub struct ChatUpdate {
    pub name: Option<String>,
    pub summary: Option<String>,
    pub confidence: Option<f32>,
    pub embedding: Option<Vec<f32>>,
}

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: ChatId, value: ChatUpdate) -> Result<Chat, Error> {
        let mut row = self.storage.chats().find_by_id(id.into()).await?;

        if let Some(value) = value.name {
            row.name = value;
        }

        if let Some(value) = value.summary {
            row.summary = Some(value);
        }

        if let Some(value) = value.confidence {
            row.confidence = value;
        }

        if let Some(value) = value.embedding {
            row.embedding = Some(value.into());
        }

        Ok(self.storage.chats().update(row).await?.into())
    }
}
