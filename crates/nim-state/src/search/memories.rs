use nim_store::{MemoryFilter, SearchItem};

use crate::{Error, Memory, SearchService};

impl<'a> SearchService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn memories(&self, embedding: &[f32], filter: MemoryFilter) -> Result<Vec<SearchItem<Memory>>, Error> {
        let rows = self.storage.search().memories(embedding, filter).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                SearchItem::new(
                    row.score,
                    Memory {
                        id: row.id.into(),
                        version: row.version.into(),
                        ty: row.ty.into(),
                        contexts: row.contexts.iter().map(|c| c.clone().into()).collect(),
                        sources: row.sources.iter().map(|v| (*v).into()).collect(),
                        artifacts: Default::default(),
                        relations: Default::default(),
                        salience: row.salience,
                        strength: row.strength,
                        confidence: row.confidence,
                        recalls: row.recalls as u64,
                        recalled_at: row.recalled_at,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                    },
                )
            })
            .collect())
    }
}
