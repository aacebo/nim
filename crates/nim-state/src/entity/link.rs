use nim_store::EntityMemoryRow;

use crate::{EntityId, EntityMemory, EntityService, Error, MemoryId};

impl<'a> EntityService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn link(&self, memory_id: MemoryId, entity_id: EntityId) -> Result<EntityMemory, Error> {
        let row = self
            .storage
            .entity_memories()
            .create(EntityMemoryRow {
                entity_id: entity_id.into(),
                memory_id: memory_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }
}
