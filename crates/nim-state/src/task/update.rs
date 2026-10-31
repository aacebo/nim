use crate::{Error, Task, TaskId, TaskService, TaskStatus};

#[derive(Debug, Default)]
pub struct TaskUpdate {
    pub parent_id: Option<TaskId>,
    pub name: Option<String>,
    pub source: Option<String>,
    pub status: Option<TaskStatus>,
}

impl<'a> TaskService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: TaskId, value: TaskUpdate) -> Result<Task, Error> {
        let mut row = self.storage.tasks().find_by_id(id.into()).await?;

        if let Some(value) = value.parent_id {
            row.parent_id = Some(value.into());
        }

        if let Some(value) = value.name {
            row.name = value;
        }

        if let Some(value) = value.source {
            row.source = value;
        }

        if let Some(value) = value.status {
            row.status = nim_store::Json(value);
        }

        Ok(self.storage.tasks().update(row).await?.into())
    }
}
