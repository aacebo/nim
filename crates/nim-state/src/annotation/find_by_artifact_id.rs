use crate::{Annotation, AnnotationService, ArtifactId, Error};

impl<'a> AnnotationService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: ArtifactId) -> Result<Vec<Annotation>, Error> {
        Ok(self
            .store
            .find_by_artifact_id(artifact_id.into())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
