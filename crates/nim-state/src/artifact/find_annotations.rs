use crate::{ArtifactAnnotation, ArtifactId, ArtifactService};

impl<'a> ArtifactService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_annotations(&self, artifact_id: ArtifactId) -> nim_error::Result<Vec<ArtifactAnnotation>> {
        self.storage
            .artifact_annotations()
            .find_by_artifact_id(artifact_id.into())
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }
}
