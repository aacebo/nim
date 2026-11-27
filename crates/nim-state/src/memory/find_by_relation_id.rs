use crate::{Memory, MemoryId, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_relation_id(&self, relation_id: MemoryId) -> nim_error::Result<Vec<Memory>> {
        let rows = self.storage.memories().find_by_relation_id(relation_id.into()).await?;
        let mut memories = Vec::with_capacity(rows.len());

        for row in rows {
            memories.push(row.try_into()?);
        }

        Ok(memories)
    }
}
