use crate::{Error, Plan, PlanId, PlanService};

impl<'a> PlanService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: PlanId) -> Result<Plan, Error> {
        Ok(self.store.find_by_id(id.into()).await?.into())
    }
}
