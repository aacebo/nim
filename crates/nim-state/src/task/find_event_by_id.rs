use crate::{Error, TaskEvent, TaskEventId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_event_by_id(&self, id: TaskEventId) -> Result<TaskEvent, Error> {
        self.storage.task_events().find_by_id(id.into()).await?.try_into()
    }
}
