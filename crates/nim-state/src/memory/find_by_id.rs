use crate::{Error, Memory, MemoryId, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: MemoryId) -> Result<Memory, Error> {
        let row = self.store.find_by_id(id.into()).await?;

        Ok(Memory {
            id: row.id.into(),
            version: Default::default(),
            ty: row.ty.into(),
            contexts: row.contexts.iter().map(|c| c.clone().into()).collect(),
            sources: Default::default(),
            relations: Default::default(),
            annotations: Default::default(),
            salience: row.salience,
            strength: row.strength,
            confidence: row.confidence,
            recalls: row.recalls as u64,
            description: row.description,
            summary: row.summary,
            embedding: row.embedding.map(|v| v.into()),
            recalled_at: row.recalled_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
