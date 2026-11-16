use crate::{Recall, RecallId, RecallService};

impl<'a> RecallService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: RecallId) -> nim_error::Result<Recall> {
        Ok(self.storage.recalls().find_by_id(id.into()).await?.into())
    }
}
