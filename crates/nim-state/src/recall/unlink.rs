use crate::{Error, MemoryId, RecallId, RecallService};

impl<'a> RecallService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unlink(&self, memory_id: MemoryId, recall_id: RecallId) -> Result<(), Error> {
        self.storage
            .memory_recalls()
            .delete((memory_id.into(), recall_id.into()))
            .await?;

        Ok(())
    }
}
