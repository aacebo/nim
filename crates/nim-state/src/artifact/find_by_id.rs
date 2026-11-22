use crate::{Artifact, ArtifactId, ArtifactService};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: ArtifactId) -> nim_error::Result<Artifact> {
        self.storage.artifacts().find_by_id(id.into()).await?.try_into()
    }
}
