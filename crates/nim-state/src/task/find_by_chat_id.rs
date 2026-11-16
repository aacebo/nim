use crate::{ChatId, Task, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: ChatId) -> nim_error::Result<Vec<Task>> {
        let rows = self.storage.tasks().find_by_chat_id(chat_id.into()).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
