use nim_store::{MessageFilter, SearchItem};

use crate::{Error, Message, SearchService};

impl<'a> SearchService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn messages(&self, embedding: &[f32], filter: MessageFilter) -> Result<Vec<SearchItem<Message>>, Error> {
        let rows = self.storage.search().messages(embedding, filter).await?;

        rows.into_iter()
            .map(|row| Ok(SearchItem::new(row.score, row.into_inner().try_into()?)))
            .collect()
    }
}
