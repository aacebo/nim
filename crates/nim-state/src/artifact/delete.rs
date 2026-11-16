use crate::{ArtifactId, ArtifactService};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: ArtifactId) -> nim_error::Result<()> {
        Ok(self.storage.artifacts().delete(id.into()).await?)
    }
}
