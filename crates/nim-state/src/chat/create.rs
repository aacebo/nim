use nim_store::ChatRow;

use crate::{Chat, ChatService, Error};

#[derive(Debug)]
pub struct ChatCreate {
    pub name: String,
}

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: ChatCreate) -> Result<Chat, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .chats()
            .create(ChatRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                name: value.name,
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
