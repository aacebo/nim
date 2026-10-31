use crate::{Error, Task, TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: TaskId) -> Result<Task, Error> {
        Ok(self.storage.tasks().find_by_id(id.into()).await?.into())
    }
}
