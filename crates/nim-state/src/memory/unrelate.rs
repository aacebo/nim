use crate::{Error, MemoryId, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unrelate(&self, source_id: MemoryId, target_id: MemoryId) -> Result<(), Error> {
        self.storage
            .memory_relations()
            .delete((source_id.into(), target_id.into()))
            .await?;

        Ok(())
    }
}
