use crate::{Error, ModelId, Task, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_model_id(&self, model_id: ModelId) -> Result<Vec<Task>, Error> {
        let rows = self.storage.tasks().find_by_model_id(model_id.into()).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
