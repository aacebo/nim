use nim_store::{Filter, SearchItem};

use crate::{Artifact, SearchService};

impl<'a> SearchService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn artifacts(&self, embedding: &[f32], filter: Filter) -> nim_error::Result<Vec<SearchItem<Artifact>>> {
        let rows = self.storage.search().artifacts(embedding, filter).await?;

        rows.into_iter()
            .map(|row| Ok(SearchItem::new(row.score, row.into_inner().try_into()?)))
            .collect()
    }
}
