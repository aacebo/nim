use crate::{Task, TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: TaskId) -> nim_error::Result<Task> {
        self.storage.tasks().find_by_id(id.into()).await?.try_into()
    }
}
