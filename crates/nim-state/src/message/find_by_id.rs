use crate::{Error, Message, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: MessageId) -> Result<Message, Error> {
        Ok(self.storage.messages().find_by_id(id.into()).await?.into())
    }
}
