use crate::{Chat, ChatId, ChatService, Error, MessageId};

#[derive(Debug, Default)]
pub struct ChatUpdate {
    pub summary: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub confidence: Option<f32>,
    pub last_analyzed_id: Option<MessageId>,
}

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: ChatId, value: ChatUpdate) -> Result<Chat, Error> {
        let mut row = self.storage.chats().find_by_id(id.into()).await?;

        if let Some(value) = value.summary {
            row.summary = Some(value);
        }

        if let Some(value) = value.embedding {
            row.embedding = Some(value.into());
        }

        if let Some(value) = value.confidence {
            row.confidence = value;
        }

        if let Some(value) = value.last_analyzed_id {
            row.last_analyzed_id = Some(value.into());
        }

        Ok(self.storage.chats().update(row).await?.into())
    }
}
