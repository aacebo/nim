use crate::{TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: TaskId) -> nim_error::Result<()> {
        Ok(self.storage.tasks().delete(id.into()).await?)
    }
}
