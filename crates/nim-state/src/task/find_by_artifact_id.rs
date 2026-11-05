use crate::{ArtifactId, Error, Task, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_artifact_id(&self, artifact_id: ArtifactId) -> Result<Vec<Task>, Error> {
        let rows = self.storage.tasks().find_by_artifact_id(artifact_id.into()).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
