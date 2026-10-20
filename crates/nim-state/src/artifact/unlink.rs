use crate::{ArtifactId, ArtifactService, Error, MemoryId};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unlink(&self, memory_id: MemoryId, artifact_id: ArtifactId) -> Result<(), Error> {
        self.store.unlink(memory_id.into(), artifact_id.into()).await?;

        Ok(())
    }
}
