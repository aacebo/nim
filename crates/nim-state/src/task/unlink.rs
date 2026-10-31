use crate::{ArtifactId, Error, ModelId, TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unlink_model(&self, model_id: ModelId, task_id: TaskId) -> Result<(), Error> {
        self.storage.model_tasks().delete((model_id.into(), task_id.into())).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn unlink_artifact(&self, artifact_id: ArtifactId, task_id: TaskId) -> Result<(), Error> {
        self.storage
            .artifact_tasks()
            .delete((artifact_id.into(), task_id.into()))
            .await?;

        Ok(())
    }
}
