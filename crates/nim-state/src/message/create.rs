use nim_store::MessageRow;

use crate::{ChatId, Error, Message, MessageContent, MessageService, Role};

#[derive(Debug)]
pub struct MessageCreate {
    pub chat_id: ChatId,
    pub role: Role,
    pub content: Vec<MessageContent>,
}

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: MessageCreate) -> Result<Message, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .messages()
            .create(MessageRow {
                id: uuid::Uuid::now_v7(),
                chat_id: value.chat_id.into(),
                version: 0,
                role: value.role,
                content: nim_store::Json(value.content),
                summary: None,
                confidence: 0.0,
                embedding: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
