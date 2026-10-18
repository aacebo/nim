use crate::{Error, Plan, PlanId, PlanService, PlanStatus};

#[derive(Debug, Default)]
pub struct PlanUpdate {
    pub title: Option<String>,
    pub status: Option<PlanStatus>,
    pub description: Option<String>,
}

impl<'a> PlanService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: PlanId, value: PlanUpdate) -> Result<Plan, Error> {
        let mut row = self.store.find_by_id(id.into()).await?;

        if let Some(value) = value.title {
            row.title = value;
        }

        if let Some(value) = value.status {
            row.status = value.into();
        }

        if let Some(value) = value.description {
            row.description = value;
        }

        Ok(self.store.update(row).await?.into())
    }
}
