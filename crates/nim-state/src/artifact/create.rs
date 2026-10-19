use nim_store::ArtifactRow;

use crate::{Artifact, ArtifactContent, ArtifactService, Error, MemoryId};

#[derive(Debug)]
pub struct ArtifactCreate {
    pub memory_id: MemoryId,
    pub name: Option<String>,
    pub content: ArtifactContent,
}

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self, value))]
    pub async fn create(&self, value: ArtifactCreate) -> Result<Artifact, Error> {
        let now = chrono::Utc::now();
        let row = self
            .store
            .create(ArtifactRow {
                id: uuid::Uuid::now_v7(),
                memory_id: value.memory_id.into(),
                name: value.name,
                content: nim_store::Json(value.content),
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
