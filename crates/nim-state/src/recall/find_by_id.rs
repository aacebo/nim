use crate::{Error, Recall, RecallId, RecallService};

impl<'a> RecallService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: RecallId) -> Result<Recall, Error> {
        Ok(self.store.find_by_id(id.into()).await?.into())
    }
}
