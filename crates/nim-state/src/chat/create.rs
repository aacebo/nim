use nim_store::ChatRow;

use crate::{Chat, ChatService, Error};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self) -> Result<Chat, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .chats()
            .create(ChatRow {
                id: uuid::Uuid::now_v7(),
                version: 0,
                summary: None,
                embedding: None,
                confidence: 0.0,
                last_analyzed_id: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
