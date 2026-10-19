use crate::{ArtifactId, ArtifactService, Error};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: ArtifactId) -> Result<(), Error> {
        Ok(self.store.delete(id.into()).await?)
    }
}
