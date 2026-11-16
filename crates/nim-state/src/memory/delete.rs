use crate::{MemoryId, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: MemoryId) -> nim_error::Result<()> {
        Ok(self.storage.memories().delete(id.into()).await?)
    }
}
