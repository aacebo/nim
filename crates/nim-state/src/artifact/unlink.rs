use crate::{ArtifactId, ArtifactService, Error, MemoryId};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unlink(&self, memory_id: MemoryId, artifact_id: ArtifactId) -> Result<(), Error> {
        self.storage
            .memory_artifacts()
            .delete((memory_id.into(), artifact_id.into()))
            .await?;

        Ok(())
    }
}
