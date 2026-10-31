use crate::{Error, Message, MessageContent, MessageId, MessageService};

#[derive(Debug, Default)]
pub struct MessageUpdate {
    pub content: Option<Vec<MessageContent>>,
    pub summary: Option<String>,
    pub confidence: Option<f32>,
    pub embedding: Option<Vec<f32>>,
}

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: MessageId, value: MessageUpdate) -> Result<Message, Error> {
        let mut row = self.storage.messages().find_by_id(id.into()).await?;

        if let Some(value) = value.content {
            row.content = nim_store::Json(value);
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

        Ok(self.storage.messages().update(row).await?.into())
    }
}
