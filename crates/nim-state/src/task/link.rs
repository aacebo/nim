use crate::{ArtifactId, ArtifactTask, ChatId, ChatTask, Error, MessageId, MessageTask, ModelId, ModelTask, TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn link_model(&self, model_id: ModelId, task_id: TaskId) -> Result<ModelTask, Error> {
        let row = self
            .storage
            .model_tasks()
            .create(nim_store::ModelTaskRow {
                model_id: model_id.into(),
                task_id: task_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn link_artifact(&self, artifact_id: ArtifactId, task_id: TaskId) -> Result<ArtifactTask, Error> {
        let row = self
            .storage
            .artifact_tasks()
            .create(nim_store::ArtifactTaskRow {
                artifact_id: artifact_id.into(),
                task_id: task_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn link_chat(&self, chat_id: ChatId, task_id: TaskId) -> Result<ChatTask, Error> {
        let row = self
            .storage
            .chat_tasks()
            .create(nim_store::ChatTaskRow {
                chat_id: chat_id.into(),
                task_id: task_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    pub async fn link_message(&self, message_id: MessageId, task_id: TaskId) -> Result<MessageTask, Error> {
        let row = self
            .storage
            .message_tasks()
            .create(nim_store::MessageTaskRow {
                message_id: message_id.into(),
                task_id: task_id.into(),
                created_at: chrono::Utc::now(),
            })
            .await?;

        Ok(row.into())
    }
}
