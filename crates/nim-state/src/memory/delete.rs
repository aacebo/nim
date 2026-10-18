use crate::{Error, MemoryId, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: MemoryId) -> Result<(), Error> {
        Ok(self.storage.memories().delete(id.into()).await?)
    }
}
