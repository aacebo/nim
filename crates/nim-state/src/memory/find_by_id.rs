use crate::{Memory, MemoryId, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: MemoryId) -> nim_error::Result<Memory> {
        self.storage.memories().find_by_id(id.into()).await?.try_into()
    }
}
