use crate::{Error, ModelId, Task, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_model_id(&self, model_id: ModelId) -> Result<Vec<Task>, Error> {
        Ok(self
            .storage
            .tasks()
            .find_by_model_id(model_id.into())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
