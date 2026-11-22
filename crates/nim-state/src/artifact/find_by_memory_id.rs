use crate::{Artifact, ArtifactService, MemoryId};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_memory_id(&self, memory_id: MemoryId) -> nim_error::Result<Vec<Artifact>> {
        self.storage
            .artifacts()
            .find_by_memory_id(memory_id.into())
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }
}
