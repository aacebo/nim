use nim_store::ArtifactRow;

use crate::{Artifact, ArtifactContent, ArtifactService, Embedding, Error};

#[derive(Debug)]
pub struct ArtifactCreate {
    pub name: String,
    pub content: ArtifactContent,
    pub summary: Option<String>,
    pub embedding: Option<Embedding>,
}

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self, value))]
    pub async fn create(&self, value: ArtifactCreate) -> Result<Artifact, Error> {
        let now = chrono::Utc::now();
        let row = self
            .store
            .create(ArtifactRow {
                id: uuid::Uuid::now_v7(),
                name: value.name,
                content: nim_store::Json(value.content),
                summary: value.summary,
                embedding: value.embedding.map(Into::into),
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
