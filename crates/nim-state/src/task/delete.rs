use crate::{Error, TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: TaskId) -> Result<(), Error> {
        Ok(self.storage.tasks().delete(id.into()).await?)
    }
}
