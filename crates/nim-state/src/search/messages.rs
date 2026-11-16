use nim_store::{MessageFilter, SearchItem};

use crate::{Message, SearchService};

impl<'a> SearchService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn messages(&self, embedding: &[f32], filter: MessageFilter) -> nim_error::Result<Vec<SearchItem<Message>>> {
        let rows = self.storage.search().messages(embedding, filter).await?;

        rows.into_iter()
            .map(|row| Ok(SearchItem::new(row.score, row.into_inner().try_into()?)))
            .collect()
    }
}
