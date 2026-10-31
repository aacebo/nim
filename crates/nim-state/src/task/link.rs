use crate::{ArtifactId, ArtifactTask, Error, ModelId, ModelTask, TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn link_model(&self, model_id: ModelId, task_id: TaskId) -> Result<ModelTask, Error> {
        let row = self
            .storage
            .model_tasks()
            .create(nim_store::ModelTaskRow {
                model_id: model_id.into(),
                task_id: task_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn link_artifact(&self, artifact_id: ArtifactId, task_id: TaskId) -> Result<ArtifactTask, Error> {
        let row = self
            .storage
            .artifact_tasks()
            .create(nim_store::ArtifactTaskRow {
                artifact_id: artifact_id.into(),
                task_id: task_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }
}
