use crate::{MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: MessageId) -> nim_error::Result<()> {
        Ok(self.storage.messages().delete(id.into()).await?)
    }
}
