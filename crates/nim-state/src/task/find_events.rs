use crate::{Error, TaskEvent, TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_events(&self, task_id: TaskId) -> Result<Vec<TaskEvent>, Error> {
        let rows = self.storage.task_events().find_by_task_id(task_id.into()).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
