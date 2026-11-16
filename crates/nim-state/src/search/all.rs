use nim_store::{Filter, SearchItem};

use crate::{Record, SearchService};

impl<'a> SearchService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn all(&self, embedding: &[f32], filter: Filter) -> nim_error::Result<Vec<SearchItem<Record>>> {
        let rows = self.storage.search().all(embedding, filter).await?;

        rows.into_iter()
            .map(|row| Ok(SearchItem::new(row.score, row.into_inner().try_into()?)))
            .collect()
    }
}
