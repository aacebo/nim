use crate::{AnnotationId, ArtifactId, ArtifactService, Error};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete_annotation(&self, artifact_id: ArtifactId, id: AnnotationId) -> Result<(), Error> {
        Ok(self
            .storage
            .artifact_annotations()
            .delete(artifact_id.into(), id.into())
            .await?)
    }
}
