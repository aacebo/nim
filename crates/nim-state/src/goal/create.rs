use nim_store::GoalRow;

use crate::{Error, Goal, GoalCondition, GoalId, GoalService};

#[derive(Debug)]
pub struct GoalCreate<'a> {
    pub parent_id: Option<GoalId>,
    pub description: String,
    pub priority: f32,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub conditions: &'a [GoalCondition],
}

impl<'a> GoalService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create<'b>(&self, value: GoalCreate<'b>) -> Result<Goal, Error> {
        let now = chrono::Utc::now();
        let row = self
            .store
            .create(GoalRow {
                id: uuid::Uuid::now_v7(),
                parent_id: value.parent_id.map(Into::into),
                version: 0,
                description: value.description,
                status: nim_store::GoalStatus::Active,
                priority: value.priority as f64,
                deadline: value.deadline,
                conditions: nim_store::Json(value.conditions.iter().cloned().map(Into::into).collect()),
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
