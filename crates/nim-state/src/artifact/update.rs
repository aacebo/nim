use crate::{Artifact, ArtifactContent, ArtifactId, ArtifactService, Error};

#[derive(Debug, Default)]
pub struct ArtifactUpdate {
    pub name: Option<String>,
    pub content: Option<ArtifactContent>,
}

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self, value))]
    pub async fn update(&self, id: ArtifactId, value: ArtifactUpdate) -> Result<Artifact, Error> {
        let mut row = self.store.find_by_id(id.into()).await?;

        if let Some(value) = value.name {
            row.name = value;
        }

        if let Some(value) = value.content {
            row.content = nim_store::Json(value);
        }

        Ok(self.store.update(row).await?.into())
    }
}
