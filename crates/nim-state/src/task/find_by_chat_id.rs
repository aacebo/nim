use crate::{ChatId, Error, Task, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: ChatId) -> Result<Vec<Task>, Error> {
        Ok(self
            .storage
            .tasks()
            .find_by_chat_id(chat_id.into())
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
