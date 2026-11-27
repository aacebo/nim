use crate::{ArtifactId, Memory, MemoryService};

impl<'a> MemoryService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: ArtifactId) -> nim_error::Result<Vec<Memory>> {
        let rows = self.storage.memories().find_by_artifact_id(artifact_id.into()).await?;
        let mut memories = Vec::with_capacity(rows.len());

        for row in rows {
            memories.push(row.try_into()?);
        }

        Ok(memories)
    }
}
