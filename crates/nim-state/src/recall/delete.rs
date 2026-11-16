use crate::{RecallId, RecallService};

impl<'a> RecallService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: RecallId) -> nim_error::Result<()> {
        Ok(self.storage.recalls().delete(id.into()).await?)
    }
}
