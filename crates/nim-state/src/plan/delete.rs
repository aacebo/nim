use crate::{Error, PlanId, PlanService};

impl<'a> PlanService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: PlanId) -> Result<(), Error> {
        Ok(self.store.delete(id.into()).await?)
    }
}
