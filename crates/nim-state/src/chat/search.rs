use nim_store::{ChatFilter, SearchItem};

use crate::{Chat, ChatService, Error};

impl<'a> ChatService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn search(&self, embedding: &[f32], filter: ChatFilter) -> Result<Vec<SearchItem<Chat>>, Error> {
        let rows = self.storage.chats().search(embedding, filter).await?;

        Ok(rows
            .into_iter()
            .map(|row| SearchItem::new(row.score, row.into_inner().into()))
            .collect())
    }
}
