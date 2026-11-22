use nim_store::{MemoryFilter, SearchItem};

use crate::{Memory, SearchService};

impl<'a> SearchService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn memories(&self, embedding: &[f32], filter: MemoryFilter) -> nim_error::Result<Vec<SearchItem<Memory>>> {
        let rows = self.storage.search().memories(embedding, filter).await?;

        rows.into_iter()
            .map(|row| Ok(SearchItem::new(row.score, row.into_inner().try_into()?)))
            .collect()
    }
}
