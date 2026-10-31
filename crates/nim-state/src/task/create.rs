use nim_store::TaskRow;

use crate::{Error, Task, TaskId, TaskService, TaskStatus};

#[derive(Debug)]
pub struct TaskCreate {
    pub parent_id: Option<TaskId>,
    pub name: String,
    pub source: String,
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
                source: value.source,
                status: nim_store::Json(TaskStatus::Pending),
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
