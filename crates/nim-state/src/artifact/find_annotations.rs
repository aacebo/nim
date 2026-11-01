use crate::{ArtifactAnnotation, ArtifactId, ArtifactService, Error};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotations(&self, artifact_id: ArtifactId) -> Result<Vec<ArtifactAnnotation>, Error> {
        Ok(self
            .storage
            .artifact_annotations()
            .find_by_artifact_id(artifact_id.into())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
