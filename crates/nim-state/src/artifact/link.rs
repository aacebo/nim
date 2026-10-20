use crate::{ArtifactId, ArtifactService, Error, MemoryArtifact, MemoryId};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn link(&self, memory_id: MemoryId, artifact_id: ArtifactId) -> Result<MemoryArtifact, Error> {
        Ok(self.store.link(memory_id.into(), artifact_id.into()).await?.into())
    }
}
