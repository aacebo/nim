use nim_store::PlanRow;

use crate::{Error, Plan, PlanService};

#[derive(Debug)]
pub struct PlanCreate {
    pub title: String,
    pub description: String,
}

impl<'a> PlanService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn create(&self, value: PlanCreate) -> Result<Plan, Error> {
        let now = chrono::Utc::now();
        let row = self
            .store
            .create(PlanRow {
                id: uuid::Uuid::now_v7(),
                title: value.title,
                status: nim_store::PlanStatus::Draft,
                description: value.description,
                created_at: now,
                updated_at: now,
            })
            .await?;

        Ok(row.into())
    }
}
