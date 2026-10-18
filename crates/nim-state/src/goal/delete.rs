use crate::{Error, GoalId, GoalService};

impl<'a> GoalService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: GoalId) -> Result<(), Error> {
        Ok(self.store.delete(id.into()).await?)
    }
}
