use crate::{Artifact, ArtifactContent, ArtifactId, ArtifactService, Embedding, Error};

#[derive(Debug, Default)]
pub struct ArtifactUpdate {
    pub name: Option<String>,
    pub content: Option<ArtifactContent>,
    pub summary: Option<String>,
    pub embedding: Option<Embedding>,
}

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self, value))]
    pub async fn update(&self, id: ArtifactId, value: ArtifactUpdate) -> Result<Artifact, Error> {
        let mut row = self.storage.artifacts().find_by_id(id.into()).await?;

        if let Some(value) = value.name {
            row.name = value;
        }

        if let Some(value) = value.content {
            row.content = nim_store::Json(value);
        }

        if let Some(value) = value.summary {
            row.summary = Some(value);
        }

        if let Some(value) = value.embedding {
            row.embedding = Some(value.into());
        }

        Ok(self.storage.artifacts().update(row).await?.into())
    }
}
