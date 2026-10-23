use nim_store::SearchItem;

use crate::{Error, Memory, MemoryService, MemoryType};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn search(&self, embedding: &[f32], ty: MemoryType, limit: usize) -> Result<Vec<SearchItem<Memory>>, Error> {
        let rows = self.storage.memories().search(embedding, ty.into(), limit).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                SearchItem::new(
                    row.similarity,
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
