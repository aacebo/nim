use crate::{ArtifactId, ChatId, Error, MessageId, ModelId, TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn unlink_model(&self, model_id: ModelId, task_id: TaskId) -> Result<(), Error> {
        self.storage.model_tasks().delete((model_id.into(), task_id.into())).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn unlink_artifact(&self, artifact_id: ArtifactId, task_id: TaskId) -> Result<(), Error> {
        self.storage
            .artifact_tasks()
            .delete((artifact_id.into(), task_id.into()))
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn unlink_chat(&self, chat_id: ChatId, task_id: TaskId) -> Result<(), Error> {
        self.storage.chat_tasks().delete((chat_id.into(), task_id.into())).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn unlink_message(&self, message_id: MessageId, task_id: TaskId) -> Result<(), Error> {
        self.storage
            .message_tasks()
            .delete((message_id.into(), task_id.into()))
            .await?;

        Ok(())
    }
}
