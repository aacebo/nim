use crate::{ArtifactAnnotation, ArtifactId, ArtifactService, Error, Label};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotation_by_label(
        &self,
        artifact_id: ArtifactId,
        label: Label,
        text: &str,
    ) -> Result<Option<ArtifactAnnotation>, Error> {
        Ok(self
            .storage
            .artifact_annotations()
            .find_by_artifact_label(artifact_id.into(), label.into(), text)
            .await?
            .map(Into::into))
    }
}
