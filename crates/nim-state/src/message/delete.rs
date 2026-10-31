use crate::{Error, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: MessageId) -> Result<(), Error> {
        Ok(self.storage.messages().delete(id.into()).await?)
    }
}
