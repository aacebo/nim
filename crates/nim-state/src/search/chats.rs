use nim_store::{ChatFilter, SearchItem};

use crate::{Chat, Error, SearchService};

impl<'a> SearchService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn chats(&self, embedding: &[f32], filter: ChatFilter) -> Result<Vec<SearchItem<Chat>>, Error> {
        let rows = self.storage.search().chats(embedding, filter).await?;

        Ok(rows
            .into_iter()
            .map(|row| SearchItem::new(row.score, row.into_inner().into()))
            .collect())
    }
}
