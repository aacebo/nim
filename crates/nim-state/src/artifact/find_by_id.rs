use crate::{Artifact, ArtifactId, ArtifactService, Error};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: ArtifactId) -> Result<Artifact, Error> {
        Ok(self.store.find_by_id(id.into()).await?.into())
    }
}
