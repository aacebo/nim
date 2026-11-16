use nim_store::MemoryArtifactRow;

use crate::{ArtifactId, ArtifactService, MemoryArtifact, MemoryId};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn link(&self, memory_id: MemoryId, artifact_id: ArtifactId) -> nim_error::Result<MemoryArtifact> {
        let row = self
            .storage
            .memory_artifacts()
            .create(MemoryArtifactRow {
                memory_id: memory_id.into(),
                artifact_id: artifact_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }
}
