use nim_store::TaskEventRow;

use crate::{Error, TaskEvent, TaskEventContent, TaskId, TaskService};

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create_event(&self, task_id: TaskId, content: TaskEventContent) -> Result<TaskEvent, Error> {
        let row = self
            .storage
            .task_events()
            .create(TaskEventRow {
                id: uuid::Uuid::now_v7(),
                task_id: task_id.into(),
                content: serde_json::to_value(content)?,
                created_at: chrono::Utc::now(),
            })
            .await?;

        row.try_into()
    }
}
