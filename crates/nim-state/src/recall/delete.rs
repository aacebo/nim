use crate::{Error, RecallId, RecallService};

impl<'a> RecallService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: RecallId) -> Result<(), Error> {
        Ok(self.storage.recalls().delete(id.into()).await?)
    }
}
