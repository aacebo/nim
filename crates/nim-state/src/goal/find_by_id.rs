use crate::{Error, Goal, GoalId, GoalService};

impl<'a> GoalService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: GoalId) -> Result<Goal, Error> {
        Ok(self.store.find_by_id(id.into()).await?.into())
    }
}
