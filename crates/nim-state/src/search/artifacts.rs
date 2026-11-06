use nim_store::{Filter, SearchItem};

use crate::{Artifact, Error, SearchService};

impl<'a> SearchService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn artifacts(&self, embedding: &[f32], filter: Filter) -> Result<Vec<SearchItem<Artifact>>, Error> {
        let rows = self.storage.search().artifacts(embedding, filter).await?;

        Ok(rows
            .into_iter()
            .map(|row| SearchItem::new(row.score, row.into_inner().into()))
            .collect())
    }
}
