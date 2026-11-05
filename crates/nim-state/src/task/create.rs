use nim_store::TaskRow;

use crate::{Error, Source, Task, TaskId, TaskService, TaskStatus};

#[derive(Debug)]
pub struct TaskCreate {
    pub parent_id: Option<TaskId>,
    pub name: String,
    pub source: Source,
}

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: TaskCreate) -> Result<Task, Error> {
        let now = chrono::Utc::now();
        let row = self
            .storage
            .tasks()
            .create(TaskRow {
                id: uuid::Uuid::now_v7(),
                parent_id: value.parent_id.map(Into::into),
                version: 0,
                name: value.name,
                source: nim_store::Json(value.source.into()),
                status: serde_json::to_value(TaskStatus::Pending)?,
                created_at: now,
                updated_at: now,
            })
            .await?;

        row.try_into()
    }
}
