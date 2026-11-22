use crate::{Memory, MemoryId, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: MemoryId) -> nim_error::Result<Memory> {
        let row = self.storage.memories().find_by_id(id.into()).await?;
        let artifacts = self.storage.artifacts().find_by_memory_id(id.into()).await?;
        let mut memory: Memory = row.try_into()?;

        memory.artifacts = artifacts
            .into_iter()
            .map(TryInto::try_into)
            .collect::<nim_error::Result<_>>()?;
        Ok(memory)
    }
}
