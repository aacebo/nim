use crate::{Annotation, AnnotationService, ArtifactId, Error, Label};

impl<'a> AnnotationService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_label(
        &self,
        artifact_id: ArtifactId,
        label: Label,
        text: &str,
    ) -> Result<Option<Annotation>, Error> {
        Ok(self
            .store
            .find_by_artifact_label(artifact_id.into(), label.into(), text)
            .await?
            .map(Into::into))
    }
}
