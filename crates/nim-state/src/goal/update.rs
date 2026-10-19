use crate::{Error, Goal, GoalCondition, GoalId, GoalService, GoalStatus};

#[derive(Debug, Default)]
pub struct GoalUpdate {
    pub parent_id: Option<Option<GoalId>>,
    pub description: Option<String>,
    pub status: Option<GoalStatus>,
    pub priority: Option<f32>,
    pub deadline: Option<Option<chrono::DateTime<chrono::Utc>>>,
    pub conditions: Option<Vec<GoalCondition>>,
}

impl<'a> GoalService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: GoalId, value: GoalUpdate) -> Result<Goal, Error> {
        let mut row = self.store.find_by_id(id.into()).await?;

        if let Some(value) = value.parent_id {
            row.parent_id = value.map(Into::into);
        }

        if let Some(value) = value.description {
            row.description = value;
        }

        if let Some(value) = value.status {
            row.status = value.into();
        }

        if let Some(value) = value.priority {
            row.priority = value as f64;
        }

        if let Some(value) = value.deadline {
            row.deadline = value;
        }

        if let Some(value) = value.conditions {
            row.conditions = nim_store::Json(value.into_iter().map(Into::into).collect());
        }

        row.version += 1;
        Ok(self.store.update(row).await?.into())
    }
}
