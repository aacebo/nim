use crate::{Error, Memory, MemoryId, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: MemoryId) -> Result<Memory, Error> {
        let row = self.storage.memories().find_by_id(id.into()).await?;
        let artifacts = self.storage.artifacts().find_by_memory_id(id.into()).await?;

        Ok(Memory {
            id: row.id.into(),
            version: Default::default(),
            ty: row.ty.into(),
            contexts: row.contexts.iter().map(|c| c.clone().into()).collect(),
            sources: row.sources.iter().map(|v| (*v).into()).collect(),
            artifacts: artifacts.iter().map(|v| v.clone().into()).collect(),
            relations: Default::default(),
            salience: row.salience,
            strength: row.strength,
            confidence: row.confidence,
            recalls: row.recalls as u64,
            recalled_at: row.recalled_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
