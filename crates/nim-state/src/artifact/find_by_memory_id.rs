use crate::{Artifact, ArtifactService, Error, MemoryId};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_memory_id(&self, memory_id: MemoryId) -> Result<Vec<Artifact>, Error> {
        Ok(self
            .storage
            .artifacts()
            .find_by_memory_id(memory_id.into())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
