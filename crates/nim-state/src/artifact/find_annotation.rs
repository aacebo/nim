use crate::{AnnotationId, ArtifactAnnotation, ArtifactId, ArtifactService};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotation(&self, artifact_id: ArtifactId, id: AnnotationId) -> nim_error::Result<ArtifactAnnotation> {
        self.storage
            .artifact_annotations()
            .find_by_id(artifact_id.into(), id.into())
            .await?
            .try_into()
    }
}
