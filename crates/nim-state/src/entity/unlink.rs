use crate::{EntityId, EntityService, MemoryId};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unlink(&self, memory_id: MemoryId, entity_id: EntityId) -> nim_error::Result<()> {
        self.storage
            .entity_memories()
            .delete((entity_id.into(), memory_id.into()))
            .await?;

        Ok(())
    }
}
