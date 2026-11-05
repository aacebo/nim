use crate::{Error, MessageId, Task, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_message_id(&self, message_id: MessageId) -> Result<Vec<Task>, Error> {
        let rows = self.storage.tasks().find_by_message_id(message_id.into()).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
