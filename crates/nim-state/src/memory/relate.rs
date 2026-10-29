use nim_store::MemoryRelationRow;

use crate::{Error, MemoryId, MemoryRelation, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn relate(&self, source_id: MemoryId, target_id: MemoryId) -> Result<MemoryRelation, Error> {
        let row = self
            .storage
            .memory_relations()
            .create(MemoryRelationRow {
                source_id: source_id.into(),
                target_id: target_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }
}
